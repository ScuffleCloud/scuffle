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
use crate::models::{
    EmailRegistrationRequest, EmailRegistrationRequestId, MfaWebauthnCredential, MfaWebauthnCredentialId, User, UserEmail,
    UserId,
};
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

    async fn update_user(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::UpdateUserRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::User>, tonic::Status> {
        let global = &req.global::<G>()?;
        let (_, ext, payload) = req.into_parts();
        let session = ext.session_or_err()?;

        let mut db = global.db().await.into_tonic_internal_err("failed to get db connection")?;

        let user_id: UserId = payload
            .id
            .parse()
            .map_err(|e| tonic::Status::invalid_argument(format!("invalid user ID format: {e}")))?;

        if payload.password.is_some() {
            session.is_authorized(global, session.user_id, Action::UpdateUserPassword, user_id)?;
        }
        if payload.names.is_some() {
            session.is_authorized(global, session.user_id, Action::UpdateUserNames, user_id)?;
        }
        if payload.primary_email.is_some() {
            session.is_authorized(global, session.user_id, Action::UpdateUserPrimaryEmail, user_id)?;
        }

        let user = db
            .transaction::<_, TxError, _>(move |conn| {
                async move {
                    let mut user = users::dsl::users
                        .find(user_id)
                        .select(User::as_select())
                        .first::<User>(conn)
                        .await
                        .optional()
                        .into_tonic_internal_err("failed to query user")?
                        .ok_or_else(|| {
                            tonic::Status::with_error_details(tonic::Code::NotFound, "user not found", ErrorDetails::new())
                        })?;

                    if let Some(password_update) = payload.password {
                        // Verify password
                        if let Some(password_hash) = &user.password_hash {
                            common::verify_password(
                                password_hash,
                                &password_update.current_password.require("current_password")?,
                            )?;
                        }

                        let salt = SaltString::generate(&mut argon2::password_hash::rand_core::OsRng);
                        let new_hash = Argon2::default()
                            .hash_password(password_update.new_password.as_bytes(), &salt)
                            .into_tonic_internal_err("failed to hash password")?
                            .to_string();

                        user = diesel::update(users::dsl::users)
                            .filter(users::dsl::id.eq(user_id))
                            .set(users::dsl::password_hash.eq(&new_hash))
                            .returning(User::as_returning())
                            .get_result::<User>(conn)
                            .await
                            .optional()
                            .into_tonic_internal_err("failed to update user password")?
                            .ok_or_else(|| {
                                tonic::Status::with_error_details(
                                    tonic::Code::NotFound,
                                    "user not found",
                                    ErrorDetails::new(),
                                )
                            })?;
                    }

                    if let Some(names_update) = payload.names {
                        user = diesel::update(users::dsl::users)
                            .filter(users::dsl::id.eq(user_id))
                            .set((
                                users::dsl::preferred_name.eq(&names_update.preferred_name),
                                users::dsl::first_name.eq(&names_update.first_name),
                                users::dsl::last_name.eq(&names_update.last_name),
                            ))
                            .returning(User::as_returning())
                            .get_result::<User>(conn)
                            .await
                            .optional()
                            .into_tonic_internal_err("failed to update user password")?
                            .ok_or_else(|| {
                                tonic::Status::with_error_details(
                                    tonic::Code::NotFound,
                                    "user not found",
                                    ErrorDetails::new(),
                                )
                            })?;
                    }

                    if let Some(primary_email_update) = payload.primary_email {
                        let email = common::normalize_email(&primary_email_update.primary_email);

                        let email = user_emails::dsl::user_emails
                            .filter(user_emails::dsl::email.eq(&email).and(user_emails::dsl::user_id.eq(user_id)))
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

                        user = diesel::update(users::dsl::users)
                            .filter(users::dsl::id.eq(user_id))
                            .set(users::dsl::primary_email.eq(&email))
                            .returning(User::as_returning())
                            .get_result::<User>(conn)
                            .await
                            .into_tonic_internal_err("failed to update user password")?;
                    }

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

        let user_id: UserId = payload
            .id
            .parse()
            .map_err(|e| tonic::Status::invalid_argument(format!("invalid user ID format: {e}")))?;

        let email = common::normalize_email(&payload.email);

        session.is_authorized(
            global,
            session.user_id,
            Action::CreateUserEmail,
            UserEmail {
                email: email.clone(),
                user_id,
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
                    user_id: Some(user_id),
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

        let user_id: UserId = payload
            .id
            .parse()
            .map_err(|e| tonic::Status::invalid_argument(format!("invalid user ID format: {e}")))?;

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
                                    .and(email_registration_requests::dsl::user_id.eq(user_id))
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
                        user_id,
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

        let user_id: UserId = payload
            .id
            .parse()
            .map_err(|e| tonic::Status::invalid_argument(format!("invalid user ID format: {e}")))?;

        let mut db = global.db().await.into_tonic_internal_err("failed to connect to database")?;

        let user_email = db
            .transaction::<_, TxError, _>(move |conn| {
                async move {
                    let user_email = diesel::delete(user_emails::dsl::user_emails)
                        .filter(
                            user_emails::dsl::user_id
                                .eq(user_id)
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

    async fn create_webauthn_credential(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::CreateWebauthnCredentialRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::CreateWebauthnCredentialResponse>, tonic::Status> {
        let global = &req.global::<G>()?;
        let (_, ext, payload) = req.into_parts();
        let session = ext.session_or_err()?;

        let user_id: UserId = payload
            .id
            .parse()
            .map_err(|e| tonic::Status::invalid_argument(format!("invalid user ID format: {e}")))?;

        let attestation_response = payload.response.require("response.response")?;

        let new_credential = MfaWebauthnCredential {
            id: MfaWebauthnCredentialId::new(),
            credential_id: payload.credential_id,
            user_id,
            spki_data: attestation_response.public_key.clone(),
        };

        session.is_authorized(global, session.user_id, Action::CreateWebauthnCredential, &new_credential)?;

        let mut db = global.db().await.into_tonic_internal_err("failed to get db connection")?;

        // We do not verify the attestation certificate because we want to allow all kinds of devices to register.

        let new_credential = diesel::insert_into(mfa_webauthn_credentials::dsl::mfa_webauthn_credentials)
            .values(&new_credential)
            .returning(MfaWebauthnCredential::as_returning())
            .get_result::<MfaWebauthnCredential>(&mut db)
            .await
            .into_tonic_internal_err("failed to insert webauthn credential")?;

        Ok(tonic::Response::new(new_credential.into()))
    }

    async fn complete_create_webauthn_credential(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::CompleteCreateWebauthnCredentialRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::WebauthnCredential>, tonic::Status> {
        todo!()
    }

    async fn list_webauthn_credentials(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::UserByIdRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::WebauthnCredentialsList>, tonic::Status> {
        let global = &req.global::<G>()?;
        let (_, ext, payload) = req.into_parts();
        let session = ext.session_or_err()?;

        let mut db = global.db().await.into_tonic_internal_err("failed to get db connection")?;

        let user_id: UserId = payload
            .id
            .parse()
            .map_err(|e| tonic::Status::invalid_argument(format!("invalid user ID format: {e}")))?;

        session.is_authorized(global, session.user_id, Action::ListWebauthnCredentials, user_id)?;

        let credentials = mfa_webauthn_credentials::dsl::mfa_webauthn_credentials
            .filter(mfa_webauthn_credentials::dsl::user_id.eq(user_id))
            .select(MfaWebauthnCredential::as_select())
            .load::<MfaWebauthnCredential>(&mut db)
            .await
            .into_tonic_internal_err("failed to query webauthn credentials")?;

        Ok(tonic::Response::new(pb::scufflecloud::core::v1::WebauthnCredentialsList {
            credentials: credentials.into_iter().map(Into::into).collect(),
        }))
    }

    async fn delete_webauthn_credential(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::DeleteWebauthnCredentialRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::WebauthnCredential>, tonic::Status> {
        let global = &req.global::<G>()?;
        let (_, ext, payload) = req.into_parts();
        let session = ext.session_or_err()?;

        let mut db = global.db().await.into_tonic_internal_err("failed to get db connection")?;

        let user_id: UserId = payload
            .user_id
            .parse()
            .map_err(|e| tonic::Status::invalid_argument(format!("invalid user ID format: {e}")))?;

        let credential_id: MfaWebauthnCredentialId = payload
            .id
            .parse()
            .map_err(|e| tonic::Status::invalid_argument(format!("invalid webauthn credential ID format: {e}")))?;

        session.is_authorized(global, session.user_id, Action::DeleteWebauthnCredential, credential_id)?;

        let credential = diesel::delete(mfa_webauthn_credentials::dsl::mfa_webauthn_credentials)
            .filter(
                mfa_webauthn_credentials::dsl::id
                    .eq(credential_id)
                    .and(mfa_webauthn_credentials::dsl::user_id.eq(user_id)),
            )
            .returning(MfaWebauthnCredential::as_select())
            .get_result::<MfaWebauthnCredential>(&mut db)
            .await
            .into_tonic_internal_err("failed to delete webauthn credential")?;

        Ok(tonic::Response::new(credential.into()))
    }

    async fn create_totp_credential(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::UserByIdRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::NewTotpCredential>, tonic::Status> {
        todo!()
    }

    async fn list_totp_credentials(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::UserByIdRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::TotpCredentialsList>, tonic::Status> {
        todo!()
    }

    async fn delete_totp_credential(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::DeleteTotpCredentialRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::TotpCredential>, tonic::Status> {
        todo!()
    }

    async fn regenerate_recovery_codes(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::UserByIdRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::RecoveryCodes>, tonic::Status> {
        todo!()
    }

    async fn delete_user(
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

        session.is_authorized(global, session.user_id, Action::DeleteUser, user_id)?;

        let user = diesel::delete(users::dsl::users)
            .filter(users::dsl::id.eq(user_id))
            .returning(User::as_select())
            .get_result::<User>(&mut db)
            .await
            .into_tonic_internal_err("failed to delete webauthn credential")?;

        Ok(tonic::Response::new(user.into()))
    }
}
