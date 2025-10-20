use std::borrow::Cow;

use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use core_db_types::models::{User, UserEmail};
use core_db_types::schema::{user_emails, users};
use core_pb::v1::user_service_server::UserService;
use core_pb::v1::{User as PbUser, UserGetRequest, UserGetResponse, UserUpdateRequest, UserUpdateResponse};
use core_traits::DatabaseExt;
use diesel::query_dsl::methods::FindDsl;
use diesel::{OptionalExtension, SelectableHelper};
use diesel_async::RunQueryDsl;
use ext_traits::{OptionExt, RequestExt, ResultExt};
use tonic::async_trait;
use tonic_types::{ErrorDetails, StatusExt};

use crate::auth_session::{AuthSessionExt, AuthSessionResultExt};

fn user_to_proto(user: User) -> PbUser {
    PbUser {
        id: user.id.to_string(),
        name: Some(core_pb::v1::user::Name {
            first: user.first_name.unwrap_or_default(),
            last: user.last_name,
            preferred: user.preferred_name,
        }),
        primary_email: user.primary_email,
        avatar_url: user.avatar_url,
        created_at: Some(user.id.datetime().into()),
    }
}

#[derive(Debug, Clone, Default, diesel::AsChangeset, PartialEq, Eq)]
#[diesel(table_name = core_db_types::schema::users)]
struct UserUpdate<'a> {
    preferred_name: Option<Cow<'a, str>>,
    first_name: Option<Cow<'a, str>>,
    last_name: Option<Cow<'a, str>>,
    password_hash: Option<Cow<'a, str>>,
    primary_email: Option<Cow<'a, str>>,
}

#[async_trait]
impl<G: core_traits::Global> UserService for crate::services::CoreSvc<G> {
    async fn get(&self, request: tonic::Request<UserGetRequest>) -> tonic::Result<tonic::Response<UserGetResponse>> {
        let global = request.global::<G>()?;

        // Get authenticated user session
        let auth_session = request.auth_user::<G>().await?.required()?;

        // Get user from database
        let user = global
            .user_loader()
            .load(auth_session.id())
            .await
            .unwrap_or_default()
            .into_tonic_internal_err("failed to query user")?;

        Ok(tonic::Response::new(UserGetResponse {
            user: Some(user_to_proto(user)),
        }))
    }

    async fn update(
        &self,
        request: tonic::Request<UserUpdateRequest>,
    ) -> tonic::Result<tonic::Response<UserUpdateResponse>> {
        let global = request.global::<G>()?;

        // Get authenticated user session (must be done before into_inner())
        let auth_session = request.auth_user::<G>().await?.required()?;

        let req = request.into_inner();

        if req.password.is_some() || req.primary_email.is_some() {
            auth_session.mfa_required()?;
        }

        let mut db = global
            .db()
            .await
            .into_tonic_internal_err("failed to get database connection")?;

        let user = db
            .tx(async |tx| {
                let mut update = UserUpdate::default();

                if let Some(name) = req.name {
                    update.first_name = Some(Cow::Owned(name.first));
                    update.last_name = name.last.map(Cow::Owned);
                    update.preferred_name = name.preferred.map(Cow::Owned);
                }

                // Update password if provided
                if let Some(password) = req.password {
                    let salt = SaltString::generate(&mut rand::rngs::OsRng);
                    let argon2 = Argon2::default();
                    let password_hash = argon2
                        .hash_password(password.new_password.as_bytes(), &salt)
                        .into_tonic_internal_err("failed to hash password")?
                        .to_string();

                    let user = users::table
                        .find(auth_session.id())
                        .first::<User>(tx)
                        .await
                        .into_tonic_internal_err("failed to query user")?;

                    match (user.password_hash, password.current_password) {
                        (Some(current_hash), Some(current_password)) => {
                            let current_hash = PasswordHash::new(&current_hash)
                                .into_tonic_internal_err("failed to parse current password hash")?;

                            argon2
                                .verify_password(current_password.as_bytes(), &current_hash)
                                .map_err(|_| {
                                    tonic::Status::with_error_details(
                                        tonic::Code::InvalidArgument,
                                        "invalid current password",
                                        ErrorDetails::with_bad_request_violation(
                                            "password.current_password",
                                            "invalid current password",
                                        ),
                                    )
                                })?;
                        }
                        (None, None) => {}
                        _ => {
                            return Err(tonic::Status::with_error_details(
                                tonic::Code::InvalidArgument,
                                "invalid current password",
                                ErrorDetails::with_bad_request_violation(
                                    "password.current_password",
                                    "invalid current password",
                                ),
                            ));
                        }
                    }

                    update.password_hash = Some(Cow::Owned(password_hash));
                }

                // Update primary email if provided
                if let Some(primary_email) = req.primary_email {
                    let user_email = user_emails::table
                        .find(&primary_email)
                        .first::<UserEmail>(tx)
                        .await
                        .optional()
                        .into_tonic_internal_err("failed to query user email")?;
                    if user_email.is_none_or(|e| e.user_id != auth_session.id()) {
                        return Err(tonic::Status::with_error_details(
                            tonic::Code::InvalidArgument,
                            "invalid primary email",
                            ErrorDetails::with_bad_request_violation(
                                "primary_email",
                                "primary email is not associated with the user",
                            ),
                        ));
                    }

                    update.primary_email = Some(Cow::Owned(primary_email));
                }

                if update == UserUpdate::default() {
                    return Err(tonic::Status::with_error_details(
                        tonic::Code::InvalidArgument,
                        "no updates provided",
                        ErrorDetails::with_bad_request_violation(".", "no updates provided"),
                    ));
                }

                let user = diesel::update(users::table.find(auth_session.id()))
                    .set(update)
                    .returning(User::as_select())
                    .get_result::<User>(tx)
                    .await
                    .into_tonic_internal_err("failed to update user")?;

                Ok(user)
            })
            .await?;

        Ok(tonic::Response::new(UserUpdateResponse {
            user: Some(user_to_proto(user)),
        }))
    }
}
