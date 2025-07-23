use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use diesel::{BoolExpressionMethods, ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::scoped_futures::ScopedFutureExt;
use diesel_async::{AsyncConnection, RunQueryDsl};
use tonic::Code;
use tonic_types::{ErrorDetails, StatusExt};

use crate::cedar::Action;
use crate::common::TxError;
use crate::http_ext::RequestExt;
use crate::models::{EmailRegistrationRequest, EmailRegistrationRequestId, MfaWebauthnCredential, User, UserEmail, UserId};
use crate::schema::{email_registration_requests, mfa_webauthn_credentials, user_emails, users};
use crate::services::CoreSvc;
use crate::std_ext::{OptionExt, ResultExt};
use crate::{CoreConfig, common};

#[async_trait::async_trait]
impl<G: CoreConfig> pb::scufflecloud::core::v1::users_service_server::UsersService for CoreSvc<G> {
    async fn get_user(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::UserByIdRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::User>, tonic::Status> {
        let global = &req.global::<G>()?;
        let (_, ext, payload) = req.into_parts();
        let session = ext.session_or_err()?;

        let mut db = global.db().await.into_tonic_internal_err("failed to get db connection")?;

        let user_id: UserId = payload
            .id
            .parse()
            .map_err(|e| tonic::Status::invalid_argument(format!("invalid user ID format: {e}")))?;

        session.is_authorized(global, session.user_id, Action::GetUser, user_id)?;

        let user = users::dsl::users
            .find(user_id)
            .select(User::as_select())
            .first::<User>(&mut db)
            .await
            .optional()
            .into_tonic_internal_err("failed to query user")?
            .ok_or_else(|| {
                tonic::Status::with_error_details(tonic::Code::NotFound, "user not found", ErrorDetails::new())
            })?;

        Ok(tonic::Response::new(user.into()))
    }

    async fn update_user_password(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::UpdateUserPasswordRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::User>, tonic::Status> {
        let global = &req.global::<G>()?;
        let (_, ext, payload) = req.into_parts();
        let session = ext.session_or_err()?;

        let mut db = global.db().await.into_tonic_internal_err("failed to get db connection")?;

        let user_id: UserId = payload
            .id
            .parse()
            .map_err(|e| tonic::Status::invalid_argument(format!("invalid user ID format: {e}")))?;

        session.is_authorized(global, session.user_id, Action::UpdateUserPassword, user_id)?;

        let salt = SaltString::generate(&mut argon2::password_hash::rand_core::OsRng);
        let new_hash = Argon2::default()
            .hash_password(payload.new_password.as_bytes(), &salt)
            .into_tonic_internal_err("failed to hash password")?
            .to_string();

        let user = db
            .transaction::<_, TxError, _>(move |conn| {
                async move {
                    let user = users::dsl::users
                        .find(user_id)
                        .select(User::as_select())
                        .first::<User>(conn)
                        .await
                        .optional()
                        .into_tonic_internal_err("failed to query user")?
                        .ok_or_else(|| {
                            tonic::Status::with_error_details(tonic::Code::NotFound, "user not found", ErrorDetails::new())
                        })?;

                    // Verify password
                    if let Some(password_hash) = &user.password_hash {
                        common::verify_password(password_hash, &payload.current_password.require("current_password")?)?;
                    }

                    let user = diesel::update(users::dsl::users)
                        .filter(users::dsl::id.eq(user_id))
                        .set(users::dsl::password_hash.eq(&new_hash))
                        .returning(User::as_returning())
                        .get_result::<User>(conn)
                        .await
                        .optional()
                        .into_tonic_internal_err("failed to update user password")?
                        .ok_or_else(|| {
                            tonic::Status::with_error_details(tonic::Code::NotFound, "user not found", ErrorDetails::new())
                        })?;

                    Ok(user)
                }
                .scope_boxed()
            })
            .await?;

        Ok(tonic::Response::new(user.into()))
    }

    async fn update_user_names(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::UpdateUserNamesRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::User>, tonic::Status> {
        let global = &req.global::<G>()?;
        let (_, ext, payload) = req.into_parts();
        let session = ext.session_or_err()?;

        let mut db = global.db().await.into_tonic_internal_err("failed to get db connection")?;

        let user_id: UserId = payload
            .id
            .parse()
            .map_err(|e| tonic::Status::invalid_argument(format!("invalid user ID format: {e}")))?;

        session.is_authorized(global, session.user_id, Action::UpdateUserNames, user_id)?;

        let user = diesel::update(users::dsl::users)
            .filter(users::dsl::id.eq(user_id))
            .set((
                users::dsl::preferred_name.eq(&payload.preferred_name),
                users::dsl::first_name.eq(&payload.first_name),
                users::dsl::last_name.eq(&payload.last_name),
            ))
            .returning(User::as_returning())
            .get_result::<User>(&mut db)
            .await
            .optional()
            .into_tonic_internal_err("failed to update user password")?
            .ok_or_else(|| {
                tonic::Status::with_error_details(tonic::Code::NotFound, "user not found", ErrorDetails::new())
            })?;

        Ok(tonic::Response::new(user.into()))
    }

    async fn update_user_primary_email(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::UpdateUserPrimaryEmailRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::User>, tonic::Status> {
        let global = &req.global::<G>()?;
        let (_, ext, payload) = req.into_parts();
        let session = ext.session_or_err()?;

        let mut db = global.db().await.into_tonic_internal_err("failed to get db connection")?;

        let user_id: UserId = payload
            .id
            .parse()
            .map_err(|e| tonic::Status::invalid_argument(format!("invalid user ID format: {e}")))?;

        let email = common::normalize_email(&payload.primary_email);

        session.is_authorized(global, session.user_id, Action::UpdateUserPrimaryEmail, user_id)?;

        let user = db
            .transaction::<_, TxError, _>(move |conn| {
                async move {
                    let email = user_emails::dsl::user_emails
                        .find(email)
                        .select(user_emails::dsl::email)
                        .first::<String>(conn)
                        .await
                        .optional()
                        .into_tonic_internal_err("failed to query user email")?
                        .ok_or_else(|| {
                            tonic::Status::with_error_details(
                                tonic::Code::NotFound,
                                "user email not found",
                                ErrorDetails::new(),
                            )
                        })?;

                    let user = diesel::update(users::dsl::users)
                        .filter(users::dsl::id.eq(user_id))
                        .set(users::dsl::primary_email.eq(&email))
                        .returning(User::as_returning())
                        .get_result::<User>(conn)
                        .await
                        .into_tonic_internal_err("failed to update user password")?;

                    Ok(user)
                }
                .scope_boxed()
            })
            .await?;

        Ok(tonic::Response::new(user.into()))
    }

    async fn list_user_emails(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::UserByIdRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::UserEmailsList>, tonic::Status> {
        let global = &req.global::<G>()?;
        let (_, ext, payload) = req.into_parts();
        let session = ext.session_or_err()?;

        let mut db = global.db().await.into_tonic_internal_err("failed to get db connection")?;

        let user_id: UserId = payload
            .id
            .parse()
            .map_err(|e| tonic::Status::invalid_argument(format!("invalid user ID format: {e}")))?;

        session.is_authorized(global, session.user_id, Action::ListUserEmails, user_id)?;

        let emails = user_emails::dsl::user_emails
            .filter(user_emails::dsl::user_id.eq(user_id))
            .select(UserEmail::as_select())
            .load::<UserEmail>(&mut db)
            .await
            .into_tonic_internal_err("failed to query user emails")?;

        Ok(tonic::Response::new(pb::scufflecloud::core::v1::UserEmailsList {
            emails: emails.into_iter().map(Into::into).collect(),
        }))
    }

    async fn create_user_email(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::CreateUserEmailRequest>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        let global = &req.global::<G>()?;
        let (_, ext, payload) = req.into_parts();
        let session = ext.session_or_err()?;

        let email = common::normalize_email(&payload.email);

        session.is_authorized(
            global,
            session.user_id,
            Action::CreateUserEmail,
            UserEmail {
                email: email.clone(),
                user_id: session.user_id,
                created_at: chrono::Utc::now(),
            },
        )?;

        let mut db = global.db().await.into_tonic_internal_err("failed to connect to database")?;

        // Generate random code
        let code = common::generate_random_bytes().into_tonic_internal_err("failed to generate registration code")?;
        // let code_base64 = base64::prelude::BASE64_URL_SAFE.encode(&code);

        db.transaction::<_, TxError, _>(move |conn| {
            async move {
                // Check if email is already registered
                if user_emails::dsl::user_emails
                    .find(&email)
                    .select(user_emails::dsl::email)
                    .first::<String>(conn)
                    .await
                    .optional()
                    .into_tonic_internal_err("failed to query database")?
                    .is_some()
                {
                    return Err(tonic::Status::with_error_details(
                        Code::AlreadyExists,
                        "email is already registered",
                        ErrorDetails::new(),
                    )
                    .into());
                }

                // Create email registration request
                let registration_request = EmailRegistrationRequest {
                    id: EmailRegistrationRequestId::new(),
                    user_id: Some(session.user_id),
                    email,
                    code: code.to_vec(),
                    expires_at: chrono::Utc::now() + global.email_registration_request_timeout(),
                };

                diesel::insert_into(email_registration_requests::dsl::email_registration_requests)
                    .values(registration_request)
                    .execute(conn)
                    .await
                    .into_tonic_internal_err("failed to insert email registration request")?;

                Ok(())
            }
            .scope_boxed()
        })
        .await?;

        // TODO: Send email with registration code link code_base64

        Ok(tonic::Response::new(()))
    }

    async fn complete_create_user_email(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::CompleteCreateUserEmailRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::UserEmail>, tonic::Status> {
        let global = &req.global::<G>()?;
        let (_, ext, payload) = req.into_parts();
        let session = ext.session_or_err()?;

        let mut db = global.db().await.into_tonic_internal_err("failed to connect to database")?;

        let user_email = db
            .transaction::<_, TxError, _>(move |conn| {
                async move {
                    // Delete email registration request
                    let Some(registration_request) =
                        diesel::delete(email_registration_requests::dsl::email_registration_requests)
                            .filter(
                                email_registration_requests::dsl::code
                                    .eq(&payload.code)
                                    .and(email_registration_requests::dsl::user_id.eq(session.user_id))
                                    .and(email_registration_requests::dsl::expires_at.gt(chrono::Utc::now())),
                            )
                            .returning(EmailRegistrationRequest::as_select())
                            .get_result::<EmailRegistrationRequest>(conn)
                            .await
                            .optional()
                            .into_tonic_internal_err("failed to delete email registration request")?
                    else {
                        return Err(
                            tonic::Status::with_error_details(Code::NotFound, "unknown code", ErrorDetails::new()).into(),
                        );
                    };

                    // Check if email is already registered
                    if user_emails::dsl::user_emails
                        .find(&registration_request.email)
                        .select(user_emails::dsl::email)
                        .first::<String>(conn)
                        .await
                        .optional()
                        .into_tonic_internal_err("failed to query user emails")?
                        .is_some()
                    {
                        return Err(tonic::Status::with_error_details(
                            Code::AlreadyExists,
                            "email is already registered",
                            ErrorDetails::new(),
                        )
                        .into());
                    }

                    let user_email = UserEmail {
                        email: registration_request.email,
                        user_id: session.user_id,
                        created_at: chrono::Utc::now(),
                    };

                    session.is_authorized(global, session.user_id, Action::CreateUserEmail, &user_email)?;

                    diesel::insert_into(user_emails::dsl::user_emails)
                        .values(&user_email)
                        .execute(conn)
                        .await
                        .into_tonic_internal_err("failed to insert user email")?;

                    Ok(user_email)
                }
                .scope_boxed()
            })
            .await?;

        Ok(tonic::Response::new(user_email.into()))
    }

    async fn delete_user_email(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::DeleteUserEmailRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::UserEmail>, tonic::Status> {
        let global = &req.global::<G>()?;
        let (_, ext, payload) = req.into_parts();
        let session = ext.session_or_err()?;

        let mut db = global.db().await.into_tonic_internal_err("failed to connect to database")?;

        let user_email = db
            .transaction::<_, TxError, _>(move |conn| {
                async move {
                    let user_email = diesel::delete(user_emails::dsl::user_emails)
                        .filter(
                            user_emails::dsl::user_id
                                .eq(session.user_id)
                                .and(user_emails::dsl::email.eq(&payload.email)),
                        )
                        .returning(UserEmail::as_select())
                        .get_result::<UserEmail>(conn)
                        .await
                        .into_tonic_internal_err("failed to delete user email")?;

                    session.is_authorized(global, session.user_id, Action::DeleteUserEmail, &user_email)?;

                    Ok(user_email)
                }
                .scope_boxed()
            })
            .await?;

        Ok(tonic::Response::new(user_email.into()))
    }

    async fn list_user_webauthn_credentials(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::UserByIdRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::UserWebauthnCredentialsList>, tonic::Status> {
        let global = &req.global::<G>()?;
        let (_, ext, payload) = req.into_parts();
        let session = ext.session_or_err()?;

        let mut db = global.db().await.into_tonic_internal_err("failed to get db connection")?;

        let user_id: UserId = payload
            .id
            .parse()
            .map_err(|e| tonic::Status::invalid_argument(format!("invalid user ID format: {e}")))?;

        session.is_authorized(global, session.user_id, Action::ListUserWebauthnCredentials, user_id)?;

        let credentials = mfa_webauthn_credentials::dsl::mfa_webauthn_credentials
            .filter(mfa_webauthn_credentials::dsl::user_id.eq(user_id))
            .select(MfaWebauthnCredential::as_select())
            .load::<MfaWebauthnCredential>(&mut db)
            .await
            .into_tonic_internal_err("failed to query webauthn credentials")?;

        Ok(tonic::Response::new(
            pb::scufflecloud::core::v1::UserWebauthnCredentialsList {
                credentials: credentials.into_iter().map(Into::into).collect(),
            },
        ))
    }
}
