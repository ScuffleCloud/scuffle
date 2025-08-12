use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use diesel::{BoolExpressionMethods, ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::scoped_futures::ScopedFutureExt;
use diesel_async::{AsyncConnection, RunQueryDsl};
use rand::distributions::DistString;
use tonic::Code;
use tonic_types::{ErrorDetails, StatusExt};

use crate::cedar::{self, Action, CoreApplication};
use crate::common::TxError;
use crate::http_ext::RequestExt;
use crate::models::{
    EmailRegistrationRequest, EmailRegistrationRequestId, MfaRecoveryCode, MfaRecoveryCodeId, MfaTotpCredential,
    MfaTotpCredentialId, MfaTotpRegistrationSession, MfaWebauthnAuthenticationSession, MfaWebauthnCredential,
    MfaWebauthnCredentialId, MfaWebauthnRegistrationSession, User, UserEmail, UserId,
};
use crate::schema::{
    email_registration_requests, mfa_recovery_codes, mfa_totp_credentials, mfa_totp_reg_sessions,
    mfa_webauthn_auth_sessions, mfa_webauthn_credentials, mfa_webauthn_reg_sessions, user_emails, users,
};
use crate::services::CoreSvc;
use crate::std_ext::{DisplayExt, OptionExt, ResultExt};
use crate::totp::TotpError;
use crate::{CoreConfig, common, totp};

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
        let payload = req.into_inner();

        let user_id: UserId = payload
            .id
            .parse()
            .map_err(|e| tonic::Status::invalid_argument(format!("invalid user ID: {e}")))?;

        let mut db = global.db().await.into_tonic_internal_err("failed to connect to database")?;

        let options_json = db
            .transaction::<_, TxError, _>(|conn| {
                async move {
                    let user = users::dsl::users
                        .find(user_id)
                        .select(User::as_select())
                        .first::<User>(conn)
                        .await
                        .optional()
                        .into_tonic_internal_err("failed to query user")?
                        .ok_or_else(|| {
                            tonic::Status::with_error_details(Code::NotFound, "user not found", ErrorDetails::new())
                        })?;

                    let exclude_credentials: Vec<_> = mfa_webauthn_credentials::dsl::mfa_webauthn_credentials
                        .filter(mfa_webauthn_credentials::dsl::user_id.eq(user_id))
                        .select(mfa_webauthn_credentials::dsl::credential_id)
                        .load::<Vec<u8>>(conn)
                        .await
                        .into_tonic_internal_err("failed to query webauthn credentials")?
                        .into_iter()
                        .map(webauthn_rs::prelude::CredentialID::from)
                        .collect();

                    let user_name = user.primary_email.unwrap_or(user_id.to_string());
                    let user_display_name = user.preferred_name.or_else(|| {
                        if let (Some(first_name), Some(last_name)) = (user.first_name, user.last_name) {
                            Some(format!("{} {}", first_name, last_name))
                        } else {
                            None
                        }
                    });

                    let (response, state) = global
                        .webauthn()
                        .start_passkey_registration(
                            user_id.into(),
                            &user_name,
                            user_display_name.as_ref().unwrap_or(&user_name),
                            Some(exclude_credentials),
                        )
                        .into_tonic_internal_err("failed to start webauthn registration")?;

                    let reg_session = MfaWebauthnRegistrationSession {
                        user_id,
                        state: serde_json::to_value(&state).into_tonic_internal_err("failed to serialize webauthn state")?,
                        expires_at: chrono::Utc::now() + global.mfa_timeout(),
                    };

                    let options_json =
                        serde_json::to_string(&response).into_tonic_internal_err("failed to serialize webauthn options")?;

                    diesel::insert_into(mfa_webauthn_reg_sessions::dsl::mfa_webauthn_reg_sessions)
                        .values(reg_session)
                        .execute(conn)
                        .await
                        .into_tonic_internal_err("failed to insert webauthn authentication session")?;

                    Ok(options_json)
                }
                .scope_boxed()
            })
            .await?;

        Ok(tonic::Response::new(
            pb::scufflecloud::core::v1::CreateWebauthnCredentialResponse { options_json },
        ))
    }

    async fn complete_create_webauthn_credential(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::CompleteCreateWebauthnCredentialRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::WebauthnCredential>, tonic::Status> {
        let global = &req.global::<G>()?;
        let (_, ext, payload) = req.into_parts();
        let session = ext.session_or_err()?;

        let user_id: UserId = payload
            .id
            .parse()
            .map_err(|e| tonic::Status::invalid_argument(format!("invalid user ID format: {e}")))?;

        let reg = serde_json::from_str(&payload.response_json)
            .into_tonic_err_with_field_violation("response_json", "invalid register public key credential")?;

        let mut db = global.db().await.into_tonic_internal_err("failed to get db connection")?;

        let new_credential = db
            .transaction::<_, TxError, _>(move |conn| {
                async move {
                    let state = diesel::delete(mfa_webauthn_reg_sessions::dsl::mfa_webauthn_reg_sessions)
                        .filter(
                            mfa_webauthn_reg_sessions::dsl::user_id
                                .eq(user_id)
                                .and(mfa_webauthn_reg_sessions::dsl::expires_at.gt(chrono::Utc::now())),
                        )
                        .returning(mfa_webauthn_reg_sessions::dsl::state)
                        .get_result::<serde_json::Value>(conn)
                        .await
                        .optional()
                        .into_tonic_internal_err("failed to query webauthn registration session")?
                        .ok_or_else(|| {
                            tonic::Status::with_error_details(
                                tonic::Code::FailedPrecondition,
                                "no webauthn registration session found",
                                ErrorDetails::new(),
                            )
                        })?;

                    let state: webauthn_rs::prelude::PasskeyRegistration =
                        serde_json::from_value(state).into_tonic_internal_err("failed to deserialize webauthn state")?;

                    let credential = global
                        .webauthn()
                        .finish_passkey_registration(&reg, &state)
                        .into_tonic_internal_err("failed to finish webauthn registration")?;

                    let credential = MfaWebauthnCredential {
                        id: MfaWebauthnCredentialId::new(),
                        user_id,
                        name: payload.name,
                        credential_id: credential.cred_id().to_vec(),
                        credential: serde_json::to_value(credential)
                            .into_tonic_internal_err("failed to serialize credential")?,
                        counter: None,
                        last_used_at: chrono::Utc::now(),
                    };

                    session.is_authorized(global, session.user_id, Action::CreateWebauthnCredential, &credential)?;

                    diesel::insert_into(mfa_webauthn_credentials::dsl::mfa_webauthn_credentials)
                        .values(&credential)
                        .execute(conn)
                        .await
                        .into_tonic_internal_err("failed to insert webauthn credential")?;

                    Ok(credential)
                }
                .scope_boxed()
            })
            .await?;

        Ok(tonic::Response::new(new_credential.into()))
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

    async fn create_webauthn_challenge(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::UserByIdRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::WebauthnChallenge>, tonic::Status> {
        let global = &req.global::<G>()?;
        let payload = req.into_inner();

        let user_id: UserId = payload
            .id
            .parse()
            .map_err(|e| tonic::Status::invalid_argument(format!("invalid user ID: {e}")))?;

        cedar::is_authorized(global, None, user_id, Action::LoginWithWebauthn, CoreApplication)?;

        let mut db = global.db().await.into_tonic_internal_err("failed to connect to database")?;

        let options_json = db
            .transaction::<_, TxError, _>(|conn| {
                async move {
                    let credentials = mfa_webauthn_credentials::dsl::mfa_webauthn_credentials
                        .filter(mfa_webauthn_credentials::dsl::user_id.eq(user_id))
                        .select(mfa_webauthn_credentials::dsl::credential)
                        .load::<serde_json::Value>(conn)
                        .await
                        .into_tonic_internal_err("failed to query webauthn credentials")?
                        .into_iter()
                        .map(serde_json::from_value)
                        .collect::<Result<Vec<webauthn_rs::prelude::Passkey>, _>>()
                        .into_tonic_internal_err("failed to deserialize webauthn credentials")?;

                    let (response, state) = global
                        .webauthn()
                        .start_passkey_authentication(&credentials)
                        .into_tonic_internal_err("failed to start webauthn authentication")?;

                    let auth_session = MfaWebauthnAuthenticationSession {
                        user_id,
                        state: serde_json::to_value(&state).into_tonic_internal_err("failed to serialize webauthn state")?,
                        expires_at: chrono::Utc::now() + global.mfa_timeout(),
                    };

                    let options_json =
                        serde_json::to_string(&response).into_tonic_internal_err("failed to serialize webauthn options")?;

                    diesel::insert_into(mfa_webauthn_auth_sessions::dsl::mfa_webauthn_auth_sessions)
                        .values(auth_session)
                        .execute(conn)
                        .await
                        .into_tonic_internal_err("failed to insert webauthn authentication session")?;

                    Ok(options_json)
                }
                .scope_boxed()
            })
            .await?;

        Ok(tonic::Response::new(pb::scufflecloud::core::v1::WebauthnChallenge {
            options_json,
        }))
    }

    async fn create_totp_credential(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::UserByIdRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::CreateTotpCredentialResponse>, tonic::Status> {
        let global = &req.global::<G>()?;
        let (_, ext, payload) = req.into_parts();
        ext.session_or_err()?;

        let user_id: UserId = payload
            .id
            .parse()
            .map_err(|e| tonic::Status::invalid_argument(format!("invalid user ID: {e}")))?;

        let mut db = global.db().await.into_tonic_internal_err("failed to connect to database")?;

        let response = db
            .transaction::<_, TxError, _>(|conn| {
                async move {
                    let user: User = users::dsl::users
                        .find(user_id)
                        .select(User::as_select())
                        .first(conn)
                        .await
                        .into_tonic_internal_err("failed to query user")?;

                    let totp = totp::new_token(user.primary_email.unwrap_or(user.id.to_string()))
                        .into_tonic_internal_err("failed to generate TOTP token")?;

                    let response = pb::scufflecloud::core::v1::CreateTotpCredentialResponse {
                        secret_url: totp.get_url(),
                        secret_qrcode_png: totp.get_qr_png().into_tonic_internal_err("failed to generate TOTP QR code")?,
                    };

                    diesel::insert_into(mfa_totp_reg_sessions::dsl::mfa_totp_reg_sessions)
                        .values(MfaTotpRegistrationSession {
                            user_id,
                            secret: totp.secret,
                            expires_at: chrono::Utc::now() + global.mfa_timeout(),
                        })
                        .execute(conn)
                        .await?;

                    Ok(response)
                }
                .scope_boxed()
            })
            .await?;

        Ok(tonic::Response::new(response))
    }

    async fn complete_create_totp_credential(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::CompleteCreateTotpCredentialRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::TotpCredential>, tonic::Status> {
        let global = &req.global::<G>()?;
        let (_, ext, payload) = req.into_parts();
        let session = ext.session_or_err()?;

        let user_id: UserId = payload
            .id
            .parse()
            .map_err(|e| tonic::Status::invalid_argument(format!("invalid user ID: {e}")))?;

        let mut db = global.db().await.into_tonic_internal_err("failed to connect to database")?;

        let credential = db
            .transaction::<_, TxError, _>(|conn| {
                async move {
                    let secret = mfa_totp_reg_sessions::dsl::mfa_totp_reg_sessions
                        .find(user_id)
                        .filter(mfa_totp_reg_sessions::dsl::expires_at.gt(chrono::Utc::now()))
                        .select(mfa_totp_reg_sessions::dsl::secret)
                        .first::<Vec<u8>>(conn)
                        .await
                        .optional()
                        .into_tonic_internal_err("failed to query TOTP registration session")?
                        .ok_or_else(|| {
                            tonic::Status::with_error_details(
                                tonic::Code::FailedPrecondition,
                                "no TOTP registration session found",
                                ErrorDetails::new(),
                            )
                        })?;

                    match totp::verify_token(secret.clone(), &payload.code) {
                        Ok(()) => {}
                        Err(TotpError::InvalidToken) => {
                            return Err(TotpError::InvalidToken
                                .into_tonic_err_with_field_violation("code", "invalid TOTP token")
                                .into());
                        }
                        Err(e) => return Err(e.into_tonic_internal_err("failed to verify TOTP token").into()),
                    }

                    let credential = MfaTotpCredential {
                        id: MfaTotpCredentialId::new(),
                        user_id,
                        name: payload.name,
                        secret,
                        last_used_at: chrono::Utc::now(),
                    };

                    session.is_authorized(global, session.user_id, Action::CreateTotpCredential, &credential)?;

                    diesel::insert_into(mfa_totp_credentials::dsl::mfa_totp_credentials)
                        .values(&credential)
                        .execute(conn)
                        .await
                        .into_tonic_internal_err("failed to insert TOTP credential")?;

                    Ok(credential)
                }
                .scope_boxed()
            })
            .await?;

        Ok(tonic::Response::new(credential.into()))
    }

    async fn list_totp_credentials(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::UserByIdRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::TotpCredentialsList>, tonic::Status> {
        let global = &req.global::<G>()?;
        let (_, ext, payload) = req.into_parts();
        let session = ext.session_or_err()?;

        let user_id: UserId = payload
            .id
            .parse()
            .map_err(|e| tonic::Status::invalid_argument(format!("invalid user ID: {e}")))?;

        session.is_authorized(global, session.user_id, Action::ListTotpCredentials, user_id)?;

        let mut db = global.db().await.into_tonic_internal_err("failed to connect to database")?;

        let credentials = mfa_totp_credentials::dsl::mfa_totp_credentials
            .filter(mfa_totp_credentials::dsl::user_id.eq(user_id))
            .select(MfaTotpCredential::as_select())
            .load::<MfaTotpCredential>(&mut db)
            .await
            .into_tonic_internal_err("failed to query TOTP credentials")?;

        Ok(tonic::Response::new(pb::scufflecloud::core::v1::TotpCredentialsList {
            credentials: credentials.into_iter().map(Into::into).collect(),
        }))
    }

    async fn delete_totp_credential(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::DeleteTotpCredentialRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::TotpCredential>, tonic::Status> {
        let global = &req.global::<G>()?;
        let (_, ext, payload) = req.into_parts();
        let session = ext.session_or_err()?;

        let mut db = global.db().await.into_tonic_internal_err("failed to get db connection")?;

        let user_id: UserId = payload
            .user_id
            .parse()
            .map_err(|e| tonic::Status::invalid_argument(format!("invalid user ID format: {e}")))?;

        let credential_id: MfaTotpCredentialId = payload
            .id
            .parse()
            .map_err(|e| tonic::Status::invalid_argument(format!("invalid TOTP credential ID format: {e}")))?;

        session.is_authorized(global, session.user_id, Action::DeleteTotpCredential, credential_id)?;

        let credential = diesel::delete(mfa_totp_credentials::dsl::mfa_totp_credentials)
            .filter(
                mfa_totp_credentials::dsl::id
                    .eq(credential_id)
                    .and(mfa_totp_credentials::dsl::user_id.eq(user_id)),
            )
            .returning(MfaTotpCredential::as_select())
            .get_result::<MfaTotpCredential>(&mut db)
            .await
            .into_tonic_internal_err("failed to delete TOTP credential")?;

        Ok(tonic::Response::new(credential.into()))
    }

    async fn regenerate_recovery_codes(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::UserByIdRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::RecoveryCodes>, tonic::Status> {
        let global = &req.global::<G>()?;
        let (_, ext, payload) = req.into_parts();
        let session = ext.session_or_err()?;

        let user_id: UserId = payload
            .id
            .parse()
            .map_err(|e| tonic::Status::invalid_argument(format!("invalid user ID: {e}")))?;

        session.is_authorized(global, session.user_id, Action::RegenerateRecoveryCodes, user_id)?;

        let mut db = global.db().await.into_tonic_internal_err("failed to connect to database")?;

        let mut rng = rand::rngs::OsRng;
        let codes: Vec<_> = (0..12)
            .map(|_| rand::distributions::Alphanumeric.sample_string(&mut rng, 8))
            .collect();

        let argon2 = Argon2::default();
        let recovery_codes = codes
            .iter()
            .map(|code| {
                let salt = SaltString::generate(&mut rng);
                argon2.hash_password(code.as_bytes(), &salt).map(|hash| hash.to_string())
            })
            .map(|code_hash| {
                code_hash.map(|code_hash| MfaRecoveryCode {
                    id: MfaRecoveryCodeId::new(),
                    user_id,
                    code_hash,
                })
            })
            .collect::<Result<Vec<_>, _>>()
            .into_tonic_internal_err("failed to generate recovery codes")?;

        db.transaction::<_, TxError, _>(|conn| {
            async move {
                diesel::delete(mfa_recovery_codes::dsl::mfa_recovery_codes)
                    .filter(mfa_recovery_codes::dsl::user_id.eq(user_id))
                    .execute(conn)
                    .await
                    .into_tonic_internal_err("failed to delete existing recovery codes")?;

                diesel::insert_into(mfa_recovery_codes::dsl::mfa_recovery_codes)
                    .values(recovery_codes)
                    .execute(conn)
                    .await
                    .into_tonic_internal_err("failed to insert new recovery codes")?;

                Ok(())
            }
            .scope_boxed()
        })
        .await?;

        Ok(tonic::Response::new(pb::scufflecloud::core::v1::RecoveryCodes { codes }))
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
