use argon2::Argon2;
use argon2::password_hash::{PasswordHasher, SaltString};
use base64::Engine;
use core_db_types::models::{
    MfaRecoveryCode, MfaRecoveryCodeId, MfaTotpCredential, MfaTotpCredentialId, MfaTotpRegistrationSession,
    MfaWebauthnAuthenticationSession, MfaWebauthnCredential, MfaWebauthnCredentialId, MfaWebauthnRegistrationSession,
    NewUserEmailRequest, NewUserEmailRequestId, User, UserEmail, UserId,
};
use core_db_types::schema::{
    mfa_recovery_codes, mfa_totp_credentials, mfa_totp_reg_sessions, mfa_webauthn_auth_sessions, mfa_webauthn_credentials,
    mfa_webauthn_reg_sessions, new_user_email_requests, user_emails, users,
};
use core_traits::EmailServiceClient;
use diesel::{BoolExpressionMethods, ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use ext_traits::{DisplayExt, OptionExt, RequestExt, ResultExt};
use rand::distributions::DistString;
use tonic::Code;
use tonic_types::{ErrorDetails, StatusExt};

use crate::cedar::Action;
use crate::http_ext::CoreRequestExt;
use crate::operations::{Operation, OperationDriver};
use crate::totp::TotpError;
use crate::{common, totp};

impl<G: core_traits::Global> Operation<G> for tonic::Request<pb::scufflecloud::core::v1::GetUserRequest> {
    type Principal = User;
    type Resource = User;
    type Response = pb::scufflecloud::core::v1::User;

    const ACTION: Action = Action::GetUser;

    async fn load_principal(&mut self, _driver: &mut OperationDriver<'_, G>) -> Result<Self::Principal, tonic::Status> {
        let global = &self.global::<G>()?;
        let session = self.session_or_err()?;
        common::get_user_by_id(global, session.user_id).await
    }

    async fn load_resource(&mut self, _driver: &mut OperationDriver<'_, G>) -> Result<Self::Resource, tonic::Status> {
        let global = &self.global::<G>()?;
        let user_id: UserId = self
            .get_ref()
            .id
            .parse()
            .into_tonic_err_with_field_violation("id", "invalid ID")?;

        common::get_user_by_id(global, user_id).await
    }

    async fn execute(
        self,
        _driver: &mut OperationDriver<'_, G>,
        _principal: Self::Principal,
        resource: Self::Resource,
    ) -> Result<Self::Response, tonic::Status> {
        Ok(resource.into())
    }
}

impl<G: core_traits::Global> Operation<G> for tonic::Request<pb::scufflecloud::core::v1::UpdateUserRequest> {
    type Principal = User;
    type Resource = User;
    type Response = pb::scufflecloud::core::v1::User;

    const ACTION: Action = Action::UpdateUser;

    async fn load_principal(&mut self, _driver: &mut OperationDriver<'_, G>) -> Result<Self::Principal, tonic::Status> {
        let global = &self.global::<G>()?;
        let session = self.session_or_err()?;
        common::get_user_by_id(global, session.user_id).await
    }

    async fn load_resource(&mut self, driver: &mut OperationDriver<'_, G>) -> Result<Self::Resource, tonic::Status> {
        let user_id: UserId = self
            .get_ref()
            .id
            .parse()
            .into_tonic_err_with_field_violation("id", "invalid ID")?;

        let conn = driver.conn().await?;
        common::get_user_by_id_in_tx(conn, user_id).await
    }

    async fn execute(
        self,
        driver: &mut OperationDriver<'_, G>,
        _principal: Self::Principal,
        mut resource: Self::Resource,
    ) -> Result<Self::Response, tonic::Status> {
        let payload = self.into_inner();
        let conn = driver.conn().await?;

        if let Some(password_update) = payload.password {
            // Verify password
            if let Some(password_hash) = &resource.password_hash {
                common::verify_password(password_hash, &password_update.current_password.require("current_password")?)?;
            }

            let salt = SaltString::generate(&mut argon2::password_hash::rand_core::OsRng);
            let new_hash = Argon2::default()
                .hash_password(password_update.new_password.as_bytes(), &salt)
                .into_tonic_internal_err("failed to hash password")?
                .to_string();

            resource = diesel::update(users::dsl::users)
                .filter(users::dsl::id.eq(resource.id))
                .set(users::dsl::password_hash.eq(&new_hash))
                .returning(User::as_returning())
                .get_result::<User>(conn)
                .await
                .into_tonic_internal_err("failed to update user password")?;
        }

        if let Some(names_update) = payload.names {
            resource = diesel::update(users::dsl::users)
                .filter(users::dsl::id.eq(resource.id))
                .set((
                    users::dsl::preferred_name.eq(&names_update.preferred_name),
                    users::dsl::first_name.eq(&names_update.first_name),
                    users::dsl::last_name.eq(&names_update.last_name),
                ))
                .returning(User::as_returning())
                .get_result::<User>(conn)
                .await
                .into_tonic_internal_err("failed to update user password")?;
        }

        if let Some(primary_email_update) = payload.primary_email {
            let email = common::normalize_email(&primary_email_update.primary_email);

            let email = user_emails::dsl::user_emails
                .filter(
                    user_emails::dsl::email
                        .eq(&email)
                        .and(user_emails::dsl::user_id.eq(resource.id)),
                )
                .select(user_emails::dsl::email)
                .first::<String>(conn)
                .await
                .optional()
                .into_tonic_internal_err("failed to query user email")?
                .into_tonic_not_found("user email not found")?;

            resource = diesel::update(users::dsl::users)
                .filter(users::dsl::id.eq(resource.id))
                .set(users::dsl::primary_email.eq(&email))
                .returning(User::as_returning())
                .get_result::<User>(conn)
                .await
                .into_tonic_internal_err("failed to update user password")?;
        }

        Ok(resource.into())
    }
}

impl<G: core_traits::Global> Operation<G> for tonic::Request<pb::scufflecloud::core::v1::ListUserEmailsRequest> {
    type Principal = User;
    type Resource = User;
    type Response = pb::scufflecloud::core::v1::UserEmailsList;

    const ACTION: Action = Action::ListUserEmails;

    async fn load_principal(&mut self, _driver: &mut OperationDriver<'_, G>) -> Result<Self::Principal, tonic::Status> {
        let global = &self.global::<G>()?;
        let session = self.session_or_err()?;
        common::get_user_by_id(global, session.user_id).await
    }

    async fn load_resource(&mut self, _driver: &mut OperationDriver<'_, G>) -> Result<Self::Resource, tonic::Status> {
        let global = &self.global::<G>()?;
        let user_id: UserId = self
            .get_ref()
            .id
            .parse()
            .into_tonic_err_with_field_violation("id", "invalid ID")?;

        common::get_user_by_id(global, user_id).await
    }

    async fn execute(
        self,
        _driver: &mut OperationDriver<'_, G>,
        _principal: Self::Principal,
        resource: Self::Resource,
    ) -> Result<Self::Response, tonic::Status> {
        let global = &self.global::<G>()?;
        let mut db = global.db().await.into_tonic_internal_err("failed to connect to database")?;

        let emails = user_emails::dsl::user_emails
            .filter(user_emails::dsl::user_id.eq(resource.id))
            .select(UserEmail::as_select())
            .load::<UserEmail>(&mut db)
            .await
            .into_tonic_internal_err("failed to query user emails")?;

        Ok(pb::scufflecloud::core::v1::UserEmailsList {
            emails: emails.into_iter().map(Into::into).collect(),
        })
    }
}

impl<G: core_traits::Global> Operation<G> for tonic::Request<pb::scufflecloud::core::v1::CreateUserEmailRequest> {
    type Principal = User;
    type Resource = UserEmail;
    type Response = ();

    const ACTION: Action = Action::CreateUserEmail;

    async fn load_principal(&mut self, _driver: &mut OperationDriver<'_, G>) -> Result<Self::Principal, tonic::Status> {
        let global = &self.global::<G>()?;
        let session = self.session_or_err()?;
        common::get_user_by_id(global, session.user_id).await
    }

    async fn load_resource(&mut self, _driver: &mut OperationDriver<'_, G>) -> Result<Self::Resource, tonic::Status> {
        let user_id: UserId = self
            .get_ref()
            .id
            .parse()
            .into_tonic_err_with_field_violation("id", "invalid ID")?;

        Ok(UserEmail {
            email: common::normalize_email(&self.get_ref().email),
            user_id,
            created_at: chrono::Utc::now(),
        })
    }

    async fn execute(
        self,
        driver: &mut OperationDriver<'_, G>,
        _principal: Self::Principal,
        resource: Self::Resource,
    ) -> Result<Self::Response, tonic::Status> {
        let global = &self.global::<G>()?;

        // Generate random code
        let code = common::generate_random_bytes().into_tonic_internal_err("failed to generate registration code")?;
        let code_base64 = base64::prelude::BASE64_URL_SAFE.encode(code);
        let conn = driver.conn().await?;

        // Check if email is already registered
        if user_emails::dsl::user_emails
            .find(&resource.email)
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
            ));
        }

        let user = common::get_user_by_id(global, resource.user_id).await?;

        let timeout = global.timeout_config().new_user_email_request;

        // Create email registration request
        let registration_request = NewUserEmailRequest {
            id: NewUserEmailRequestId::new(),
            user_id: resource.user_id,
            email: resource.email.clone(),
            code: code.to_vec(),
            expires_at: chrono::Utc::now() + timeout,
        };

        diesel::insert_into(new_user_email_requests::dsl::new_user_email_requests)
            .values(registration_request)
            .execute(conn)
            .await
            .into_tonic_internal_err("failed to insert email registration request")?;

        // Send email
        let email = core_emails::add_new_email_email(global.dashboard_origin(), code_base64, timeout)
            .into_tonic_internal_err("failed to render add new email email")?;
        let email = common::email_to_pb(global, resource.email.clone(), user.preferred_name, email);

        global
            .email_service()
            .send_email(email)
            .await
            .into_tonic_internal_err("failed to send add new email email")?;

        Ok(())
    }
}

impl<G: core_traits::Global> Operation<G> for tonic::Request<pb::scufflecloud::core::v1::CompleteCreateUserEmailRequest> {
    type Principal = User;
    type Resource = UserEmail;
    type Response = pb::scufflecloud::core::v1::UserEmail;

    const ACTION: Action = Action::CreateUserEmail;

    async fn load_principal(&mut self, _driver: &mut OperationDriver<'_, G>) -> Result<Self::Principal, tonic::Status> {
        let global = &self.global::<G>()?;
        let session = self.session_or_err()?;
        common::get_user_by_id(global, session.user_id).await
    }

    async fn load_resource(&mut self, driver: &mut OperationDriver<'_, G>) -> Result<Self::Resource, tonic::Status> {
        let user_id: UserId = self
            .get_ref()
            .id
            .parse()
            .into_tonic_err_with_field_violation("id", "invalid ID")?;

        let conn = driver.conn().await?;

        // Delete email registration request
        let Some(registration_request) = diesel::delete(new_user_email_requests::dsl::new_user_email_requests)
            .filter(
                new_user_email_requests::dsl::code
                    .eq(&self.get_ref().code)
                    .and(new_user_email_requests::dsl::user_id.eq(user_id))
                    .and(new_user_email_requests::dsl::expires_at.gt(chrono::Utc::now())),
            )
            .returning(NewUserEmailRequest::as_select())
            .get_result::<NewUserEmailRequest>(conn)
            .await
            .optional()
            .into_tonic_internal_err("failed to delete email registration request")?
        else {
            return Err(tonic::Status::with_error_details(
                Code::NotFound,
                "unknown code",
                ErrorDetails::new(),
            ));
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
            ));
        }

        Ok(UserEmail {
            email: registration_request.email,
            user_id,
            created_at: chrono::Utc::now(),
        })
    }

    async fn execute(
        self,
        driver: &mut OperationDriver<'_, G>,
        _principal: Self::Principal,
        resource: Self::Resource,
    ) -> Result<Self::Response, tonic::Status> {
        let conn = driver.conn().await?;

        diesel::insert_into(user_emails::dsl::user_emails)
            .values(&resource)
            .execute(conn)
            .await
            .into_tonic_internal_err("failed to insert user email")?;

        Ok(resource.into())
    }
}

impl<G: core_traits::Global> Operation<G> for tonic::Request<pb::scufflecloud::core::v1::DeleteUserEmailRequest> {
    type Principal = User;
    type Resource = UserEmail;
    type Response = pb::scufflecloud::core::v1::UserEmail;

    const ACTION: Action = Action::DeleteUserEmail;

    async fn load_principal(&mut self, _driver: &mut OperationDriver<'_, G>) -> Result<Self::Principal, tonic::Status> {
        let global = &self.global::<G>()?;
        let session = self.session_or_err()?;
        common::get_user_by_id(global, session.user_id).await
    }

    async fn load_resource(&mut self, driver: &mut OperationDriver<'_, G>) -> Result<Self::Resource, tonic::Status> {
        let user_id: UserId = self
            .get_ref()
            .id
            .parse()
            .into_tonic_err_with_field_violation("id", "invalid ID")?;

        let conn = driver.conn().await?;

        let user_email = user_emails::dsl::user_emails
            .filter(
                user_emails::dsl::user_id
                    .eq(user_id)
                    .and(user_emails::dsl::email.eq(&self.get_ref().email)),
            )
            .select(UserEmail::as_select())
            .first::<UserEmail>(conn)
            .await
            .into_tonic_internal_err("failed to delete user email")?;

        Ok(user_email)
    }

    async fn execute(
        self,
        driver: &mut OperationDriver<'_, G>,
        _principal: Self::Principal,
        resource: Self::Resource,
    ) -> Result<Self::Response, tonic::Status> {
        let conn = driver.conn().await?;

        diesel::delete(user_emails::dsl::user_emails)
            .filter(user_emails::dsl::email.eq(&resource.email))
            .execute(conn)
            .await
            .into_tonic_internal_err("failed to delete user email")?;

        Ok(resource.into())
    }
}

impl<G: core_traits::Global> Operation<G> for tonic::Request<pb::scufflecloud::core::v1::CreateWebauthnCredentialRequest> {
    type Principal = User;
    type Resource = User;
    type Response = pb::scufflecloud::core::v1::CreateWebauthnCredentialResponse;

    const ACTION: Action = Action::CreateWebauthnCredential;

    async fn load_principal(&mut self, _driver: &mut OperationDriver<'_, G>) -> Result<Self::Principal, tonic::Status> {
        let global = &self.global::<G>()?;
        let session = self.session_or_err()?;
        common::get_user_by_id(global, session.user_id).await
    }

    async fn load_resource(&mut self, driver: &mut OperationDriver<'_, G>) -> Result<Self::Resource, tonic::Status> {
        let user_id: UserId = self
            .get_ref()
            .id
            .parse()
            .into_tonic_err_with_field_violation("id", "invalid ID")?;

        let conn = driver.conn().await?;
        common::get_user_by_id_in_tx(conn, user_id).await
    }

    async fn execute(
        self,
        driver: &mut OperationDriver<'_, G>,
        _principal: Self::Principal,
        resource: Self::Resource,
    ) -> Result<Self::Response, tonic::Status> {
        let global = &self.global::<G>()?;

        let conn = driver.conn().await?;
        let exclude_credentials: Vec<_> = mfa_webauthn_credentials::dsl::mfa_webauthn_credentials
            .filter(mfa_webauthn_credentials::dsl::user_id.eq(resource.id))
            .select(mfa_webauthn_credentials::dsl::credential_id)
            .load::<Vec<u8>>(conn)
            .await
            .into_tonic_internal_err("failed to query webauthn credentials")?
            .into_iter()
            .map(webauthn_rs::prelude::CredentialID::from)
            .collect();

        let user_name = resource.primary_email.unwrap_or(resource.id.to_string());
        let user_display_name = resource.preferred_name.or_else(|| {
            if let (Some(first_name), Some(last_name)) = (resource.first_name, resource.last_name) {
                Some(format!("{} {}", first_name, last_name))
            } else {
                None
            }
        });

        let (response, state) = global
            .webauthn()
            .start_passkey_registration(
                resource.id.into(),
                &user_name,
                user_display_name.as_ref().unwrap_or(&user_name),
                Some(exclude_credentials),
            )
            .into_tonic_internal_err("failed to start webauthn registration")?;

        let reg_session = MfaWebauthnRegistrationSession {
            user_id: resource.id,
            state: serde_json::to_value(&state).into_tonic_internal_err("failed to serialize webauthn state")?,
            expires_at: chrono::Utc::now() + global.timeout_config().mfa,
        };

        let options_json =
            serde_json::to_string(&response).into_tonic_internal_err("failed to serialize webauthn options")?;

        diesel::insert_into(mfa_webauthn_reg_sessions::dsl::mfa_webauthn_reg_sessions)
            .values(&reg_session)
            .on_conflict(mfa_webauthn_reg_sessions::dsl::user_id)
            .do_update()
            .set((
                mfa_webauthn_reg_sessions::dsl::state.eq(&reg_session.state),
                mfa_webauthn_reg_sessions::dsl::expires_at.eq(&reg_session.expires_at),
            ))
            .execute(conn)
            .await
            .into_tonic_internal_err("failed to insert webauthn registration session")?;

        Ok(pb::scufflecloud::core::v1::CreateWebauthnCredentialResponse { options_json })
    }
}

impl<G: core_traits::Global> Operation<G>
    for tonic::Request<pb::scufflecloud::core::v1::CompleteCreateWebauthnCredentialRequest>
{
    type Principal = User;
    type Resource = MfaWebauthnCredential;
    type Response = pb::scufflecloud::core::v1::WebauthnCredential;

    const ACTION: Action = Action::CompleteCreateWebauthnCredential;

    async fn load_principal(&mut self, _driver: &mut OperationDriver<'_, G>) -> Result<Self::Principal, tonic::Status> {
        let global = &self.global::<G>()?;
        let session = self.session_or_err()?;
        common::get_user_by_id(global, session.user_id).await
    }

    async fn load_resource(&mut self, driver: &mut OperationDriver<'_, G>) -> Result<Self::Resource, tonic::Status> {
        let global = &self.global::<G>()?;

        let user_id: UserId = self
            .get_ref()
            .id
            .parse()
            .into_tonic_err_with_field_violation("id", "invalid ID")?;

        let reg = serde_json::from_str(&self.get_ref().response_json)
            .into_tonic_err_with_field_violation("response_json", "invalid register public key credential")?;

        let conn = driver.conn().await?;
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
            .into_tonic_err(
                tonic::Code::FailedPrecondition,
                "no webauthn registration session found",
                ErrorDetails::new(),
            )?;

        let state: webauthn_rs::prelude::PasskeyRegistration =
            serde_json::from_value(state).into_tonic_internal_err("failed to deserialize webauthn state")?;

        let credential = global
            .webauthn()
            .finish_passkey_registration(&reg, &state)
            .into_tonic_internal_err("failed to finish webauthn registration")?;

        Ok(MfaWebauthnCredential {
            id: MfaWebauthnCredentialId::new(),
            user_id,
            name: self.get_ref().name.clone(),
            credential_id: credential.cred_id().to_vec(),
            credential: serde_json::to_value(credential).into_tonic_internal_err("failed to serialize credential")?,
            counter: None,
            last_used_at: chrono::Utc::now(),
        })
    }

    async fn execute(
        self,
        driver: &mut OperationDriver<'_, G>,
        _principal: Self::Principal,
        resource: Self::Resource,
    ) -> Result<Self::Response, tonic::Status> {
        let conn = driver.conn().await?;
        diesel::insert_into(mfa_webauthn_credentials::dsl::mfa_webauthn_credentials)
            .values(&resource)
            .execute(conn)
            .await
            .into_tonic_internal_err("failed to insert webauthn credential")?;

        Ok(resource.into())
    }
}

impl<G: core_traits::Global> Operation<G> for tonic::Request<pb::scufflecloud::core::v1::ListWebauthnCredentialsRequest> {
    type Principal = User;
    type Resource = User;
    type Response = pb::scufflecloud::core::v1::WebauthnCredentialsList;

    const ACTION: Action = Action::ListWebauthnCredentials;

    async fn load_principal(&mut self, _driver: &mut OperationDriver<'_, G>) -> Result<Self::Principal, tonic::Status> {
        let global = &self.global::<G>()?;
        let session = self.session_or_err()?;
        common::get_user_by_id(global, session.user_id).await
    }

    async fn load_resource(&mut self, _driver: &mut OperationDriver<'_, G>) -> Result<Self::Resource, tonic::Status> {
        let global = &self.global::<G>()?;
        let user_id: UserId = self
            .get_ref()
            .id
            .parse()
            .into_tonic_err_with_field_violation("id", "invalid ID")?;
        common::get_user_by_id(global, user_id).await
    }

    async fn execute(
        self,
        _driver: &mut OperationDriver<'_, G>,
        _principal: Self::Principal,
        resource: Self::Resource,
    ) -> Result<Self::Response, tonic::Status> {
        let global = &self.global::<G>()?;
        let mut db = global.db().await.into_tonic_internal_err("failed to connect to database")?;

        let credentials = mfa_webauthn_credentials::dsl::mfa_webauthn_credentials
            .filter(mfa_webauthn_credentials::dsl::user_id.eq(resource.id))
            .select(MfaWebauthnCredential::as_select())
            .load::<MfaWebauthnCredential>(&mut db)
            .await
            .into_tonic_internal_err("failed to query webauthn credentials")?;

        Ok(pb::scufflecloud::core::v1::WebauthnCredentialsList {
            credentials: credentials.into_iter().map(Into::into).collect(),
        })
    }
}

impl<G: core_traits::Global> Operation<G> for tonic::Request<pb::scufflecloud::core::v1::DeleteWebauthnCredentialRequest> {
    type Principal = User;
    type Resource = MfaWebauthnCredential;
    type Response = pb::scufflecloud::core::v1::WebauthnCredential;

    const ACTION: Action = Action::DeleteWebauthnCredential;

    async fn load_principal(&mut self, _driver: &mut OperationDriver<'_, G>) -> Result<Self::Principal, tonic::Status> {
        let global = &self.global::<G>()?;
        let session = self.session_or_err()?;
        common::get_user_by_id(global, session.user_id).await
    }

    async fn load_resource(&mut self, driver: &mut OperationDriver<'_, G>) -> Result<Self::Resource, tonic::Status> {
        let user_id: UserId = self
            .get_ref()
            .user_id
            .parse()
            .into_tonic_err_with_field_violation("user_id", "invalid ID")?;

        let credential_id: MfaWebauthnCredentialId = self
            .get_ref()
            .id
            .parse()
            .into_tonic_err_with_field_violation("id", "invalid ID")?;

        let conn = driver.conn().await?;
        let credential = mfa_webauthn_credentials::dsl::mfa_webauthn_credentials
            .filter(
                mfa_webauthn_credentials::dsl::id
                    .eq(credential_id)
                    .and(mfa_webauthn_credentials::dsl::user_id.eq(user_id)),
            )
            .select(MfaWebauthnCredential::as_select())
            .first::<MfaWebauthnCredential>(conn)
            .await
            .into_tonic_internal_err("failed to delete webauthn credential")?;

        Ok(credential)
    }

    async fn execute(
        self,
        driver: &mut OperationDriver<'_, G>,
        _principal: Self::Principal,
        resource: Self::Resource,
    ) -> Result<Self::Response, tonic::Status> {
        let conn = driver.conn().await?;
        diesel::delete(mfa_webauthn_credentials::dsl::mfa_webauthn_credentials)
            .filter(mfa_webauthn_credentials::dsl::id.eq(resource.id))
            .execute(conn)
            .await
            .into_tonic_internal_err("failed to delete webauthn credential")?;

        Ok(resource.into())
    }
}

impl<G: core_traits::Global> Operation<G> for tonic::Request<pb::scufflecloud::core::v1::CreateWebauthnChallengeRequest> {
    type Principal = User;
    type Resource = User;
    type Response = pb::scufflecloud::core::v1::WebauthnChallenge;

    const ACTION: Action = Action::CreateWebauthnChallenge;

    async fn load_principal(&mut self, _driver: &mut OperationDriver<'_, G>) -> Result<Self::Principal, tonic::Status> {
        let global = &self.global::<G>()?;
        let session = self.session_or_err()?;
        common::get_user_by_id(global, session.user_id).await
    }

    async fn load_resource(&mut self, _driver: &mut OperationDriver<'_, G>) -> Result<Self::Resource, tonic::Status> {
        let global = &self.global::<G>()?;
        let user_id: UserId = self
            .get_ref()
            .id
            .parse()
            .into_tonic_err_with_field_violation("id", "invalid ID")?;
        common::get_user_by_id(global, user_id).await
    }

    async fn execute(
        self,
        driver: &mut OperationDriver<'_, G>,
        _principal: Self::Principal,
        resource: Self::Resource,
    ) -> Result<Self::Response, tonic::Status> {
        let global = &self.global::<G>()?;

        let conn = driver.conn().await?;
        let credentials = mfa_webauthn_credentials::dsl::mfa_webauthn_credentials
            .filter(mfa_webauthn_credentials::dsl::user_id.eq(resource.id))
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
            user_id: resource.id,
            state: serde_json::to_value(&state).into_tonic_internal_err("failed to serialize webauthn state")?,
            expires_at: chrono::Utc::now() + global.timeout_config().mfa,
        };

        let options_json =
            serde_json::to_string(&response).into_tonic_internal_err("failed to serialize webauthn options")?;

        diesel::insert_into(mfa_webauthn_auth_sessions::dsl::mfa_webauthn_auth_sessions)
            .values(&auth_session)
            .on_conflict(mfa_webauthn_auth_sessions::dsl::user_id)
            .do_update()
            .set((
                mfa_webauthn_auth_sessions::dsl::state.eq(&auth_session.state),
                mfa_webauthn_auth_sessions::dsl::expires_at.eq(&auth_session.expires_at),
            ))
            .execute(conn)
            .await
            .into_tonic_internal_err("failed to insert webauthn authentication session")?;

        Ok(pb::scufflecloud::core::v1::WebauthnChallenge { options_json })
    }
}

impl<G: core_traits::Global> Operation<G> for tonic::Request<pb::scufflecloud::core::v1::CreateTotpCredentialRequest> {
    type Principal = User;
    type Resource = User;
    type Response = pb::scufflecloud::core::v1::CreateTotpCredentialResponse;

    const ACTION: Action = Action::CreateTotpCredential;

    async fn load_principal(&mut self, _driver: &mut OperationDriver<'_, G>) -> Result<Self::Principal, tonic::Status> {
        let global = &self.global::<G>()?;
        let session = self.session_or_err()?;
        common::get_user_by_id(global, session.user_id).await
    }

    async fn load_resource(&mut self, driver: &mut OperationDriver<'_, G>) -> Result<Self::Resource, tonic::Status> {
        let user_id: UserId = self
            .get_ref()
            .id
            .parse()
            .into_tonic_err_with_field_violation("id", "invalid ID")?;

        let conn = driver.conn().await?;
        common::get_user_by_id_in_tx(conn, user_id).await
    }

    async fn execute(
        self,
        driver: &mut OperationDriver<'_, G>,
        _principal: Self::Principal,
        resource: Self::Resource,
    ) -> Result<Self::Response, tonic::Status> {
        let global = &self.global::<G>()?;

        let totp = totp::new_token(resource.primary_email.unwrap_or(resource.id.to_string()))
            .into_tonic_internal_err("failed to generate TOTP token")?;

        let response = pb::scufflecloud::core::v1::CreateTotpCredentialResponse {
            secret_url: totp.get_url(),
            secret_qrcode_png: totp.get_qr_png().into_tonic_internal_err("failed to generate TOTP QR code")?,
        };

        let reg_session = MfaTotpRegistrationSession {
            user_id: resource.id,
            secret: totp.secret,
            expires_at: chrono::Utc::now() + global.timeout_config().mfa,
        };

        let conn = driver.conn().await?;
        diesel::insert_into(mfa_totp_reg_sessions::dsl::mfa_totp_reg_sessions)
            .values(&reg_session)
            .on_conflict(mfa_totp_reg_sessions::dsl::user_id)
            .do_update()
            .set((
                mfa_totp_reg_sessions::dsl::secret.eq(&reg_session.secret),
                mfa_totp_reg_sessions::dsl::expires_at.eq(reg_session.expires_at),
            ))
            .execute(conn)
            .await
            .into_tonic_internal_err("failed to insert TOTP registration session")?;

        Ok(response)
    }
}

impl<G: core_traits::Global> Operation<G>
    for tonic::Request<pb::scufflecloud::core::v1::CompleteCreateTotpCredentialRequest>
{
    type Principal = User;
    type Resource = MfaTotpCredential;
    type Response = pb::scufflecloud::core::v1::TotpCredential;

    const ACTION: Action = Action::CompleteCreateTotpCredential;

    async fn load_principal(&mut self, _driver: &mut OperationDriver<'_, G>) -> Result<Self::Principal, tonic::Status> {
        let global = &self.global::<G>()?;
        let session = self.session_or_err()?;
        common::get_user_by_id(global, session.user_id).await
    }

    async fn load_resource(&mut self, driver: &mut OperationDriver<'_, G>) -> Result<Self::Resource, tonic::Status> {
        let user_id: UserId = self
            .get_ref()
            .id
            .parse()
            .into_tonic_err_with_field_violation("id", "invalid ID")?;

        let conn = driver.conn().await?;
        let secret = mfa_totp_reg_sessions::dsl::mfa_totp_reg_sessions
            .find(user_id)
            .filter(mfa_totp_reg_sessions::dsl::expires_at.gt(chrono::Utc::now()))
            .select(mfa_totp_reg_sessions::dsl::secret)
            .first::<Vec<u8>>(conn)
            .await
            .optional()
            .into_tonic_internal_err("failed to query TOTP registration session")?
            .into_tonic_err(
                tonic::Code::FailedPrecondition,
                "no TOTP registration session found",
                ErrorDetails::new(),
            )?;

        match totp::verify_token(secret.clone(), &self.get_ref().code) {
            Ok(()) => {}
            Err(TotpError::InvalidToken) => {
                return Err(TotpError::InvalidToken.into_tonic_err_with_field_violation("code", "invalid TOTP token"));
            }
            Err(e) => return Err(e.into_tonic_internal_err("failed to verify TOTP token")),
        }

        Ok(MfaTotpCredential {
            id: MfaTotpCredentialId::new(),
            user_id,
            name: self.get_ref().name.clone(),
            secret,
            last_used_at: chrono::Utc::now(),
        })
    }

    async fn execute(
        self,
        driver: &mut OperationDriver<'_, G>,
        _principal: Self::Principal,
        resource: Self::Resource,
    ) -> Result<Self::Response, tonic::Status> {
        let conn = driver.conn().await?;
        diesel::insert_into(mfa_totp_credentials::dsl::mfa_totp_credentials)
            .values(&resource)
            .execute(conn)
            .await
            .into_tonic_internal_err("failed to insert TOTP credential")?;

        Ok(resource.into())
    }
}

impl<G: core_traits::Global> Operation<G> for tonic::Request<pb::scufflecloud::core::v1::ListTotpCredentialsRequest> {
    type Principal = User;
    type Resource = User;
    type Response = pb::scufflecloud::core::v1::TotpCredentialsList;

    const ACTION: Action = Action::ListTotpCredentials;

    async fn load_principal(&mut self, _driver: &mut OperationDriver<'_, G>) -> Result<Self::Principal, tonic::Status> {
        let global = &self.global::<G>()?;
        let session = self.session_or_err()?;
        common::get_user_by_id(global, session.user_id).await
    }

    async fn load_resource(&mut self, _driver: &mut OperationDriver<'_, G>) -> Result<Self::Resource, tonic::Status> {
        let global = &self.global::<G>()?;
        let user_id: UserId = self
            .get_ref()
            .id
            .parse()
            .into_tonic_err_with_field_violation("id", "invalid ID")?;
        common::get_user_by_id(global, user_id).await
    }

    async fn execute(
        self,
        _driver: &mut OperationDriver<'_, G>,
        _principal: Self::Principal,
        resource: Self::Resource,
    ) -> Result<Self::Response, tonic::Status> {
        let global = &self.global::<G>()?;
        let mut db = global.db().await.into_tonic_internal_err("failed to connect to database")?;

        let credentials = mfa_totp_credentials::dsl::mfa_totp_credentials
            .filter(mfa_totp_credentials::dsl::user_id.eq(resource.id))
            .select(MfaTotpCredential::as_select())
            .load::<MfaTotpCredential>(&mut db)
            .await
            .into_tonic_internal_err("failed to query TOTP credentials")?;

        Ok(pb::scufflecloud::core::v1::TotpCredentialsList {
            credentials: credentials.into_iter().map(Into::into).collect(),
        })
    }
}

impl<G: core_traits::Global> Operation<G> for tonic::Request<pb::scufflecloud::core::v1::DeleteTotpCredentialRequest> {
    type Principal = User;
    type Resource = MfaTotpCredential;
    type Response = pb::scufflecloud::core::v1::TotpCredential;

    const ACTION: Action = Action::DeleteTotpCredential;

    async fn load_principal(&mut self, _driver: &mut OperationDriver<'_, G>) -> Result<Self::Principal, tonic::Status> {
        let global = &self.global::<G>()?;
        let session = self.session_or_err()?;
        common::get_user_by_id(global, session.user_id).await
    }

    async fn load_resource(&mut self, driver: &mut OperationDriver<'_, G>) -> Result<Self::Resource, tonic::Status> {
        let user_id: UserId = self
            .get_ref()
            .user_id
            .parse()
            .into_tonic_err_with_field_violation("user_id", "invalid ID")?;

        let credential_id: MfaTotpCredentialId = self
            .get_ref()
            .id
            .parse()
            .into_tonic_err_with_field_violation("id", "invalid ID")?;

        let conn = driver.conn().await?;
        let credential = mfa_totp_credentials::dsl::mfa_totp_credentials
            .filter(
                mfa_totp_credentials::dsl::id
                    .eq(credential_id)
                    .and(mfa_totp_credentials::dsl::user_id.eq(user_id)),
            )
            .select(MfaTotpCredential::as_select())
            .first::<MfaTotpCredential>(conn)
            .await
            .into_tonic_internal_err("failed to delete TOTP credential")?;

        Ok(credential)
    }

    async fn execute(
        self,
        driver: &mut OperationDriver<'_, G>,
        _principal: Self::Principal,
        resource: Self::Resource,
    ) -> Result<Self::Response, tonic::Status> {
        let conn = driver.conn().await?;
        diesel::delete(mfa_totp_credentials::dsl::mfa_totp_credentials)
            .filter(mfa_totp_credentials::dsl::id.eq(resource.id))
            .execute(conn)
            .await
            .into_tonic_internal_err("failed to delete TOTP credential")?;

        Ok(resource.into())
    }
}

impl<G: core_traits::Global> Operation<G> for tonic::Request<pb::scufflecloud::core::v1::RegenerateRecoveryCodesRequest> {
    type Principal = User;
    type Resource = User;
    type Response = pb::scufflecloud::core::v1::RecoveryCodes;

    const ACTION: Action = Action::RegenerateRecoveryCodes;

    async fn load_principal(&mut self, _driver: &mut OperationDriver<'_, G>) -> Result<Self::Principal, tonic::Status> {
        let global = &self.global::<G>()?;
        let session = self.session_or_err()?;
        common::get_user_by_id(global, session.user_id).await
    }

    async fn load_resource(&mut self, _driver: &mut OperationDriver<'_, G>) -> Result<Self::Resource, tonic::Status> {
        let global = &self.global::<G>()?;
        let user_id: UserId = self
            .get_ref()
            .id
            .parse()
            .into_tonic_err_with_field_violation("id", "invalid ID")?;
        common::get_user_by_id(global, user_id).await
    }

    async fn execute(
        self,
        driver: &mut OperationDriver<'_, G>,
        _principal: Self::Principal,
        resource: Self::Resource,
    ) -> Result<Self::Response, tonic::Status> {
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
                    user_id: resource.id,
                    code_hash,
                })
            })
            .collect::<Result<Vec<_>, _>>()
            .into_tonic_internal_err("failed to generate recovery codes")?;

        let conn = driver.conn().await?;
        diesel::delete(mfa_recovery_codes::dsl::mfa_recovery_codes)
            .filter(mfa_recovery_codes::dsl::user_id.eq(resource.id))
            .execute(conn)
            .await
            .into_tonic_internal_err("failed to delete existing recovery codes")?;

        diesel::insert_into(mfa_recovery_codes::dsl::mfa_recovery_codes)
            .values(recovery_codes)
            .execute(conn)
            .await
            .into_tonic_internal_err("failed to insert new recovery codes")?;

        Ok(pb::scufflecloud::core::v1::RecoveryCodes { codes })
    }
}

impl<G: core_traits::Global> Operation<G> for tonic::Request<pb::scufflecloud::core::v1::DeleteUserRequest> {
    type Principal = User;
    type Resource = User;
    type Response = pb::scufflecloud::core::v1::User;

    const ACTION: Action = Action::DeleteUser;

    async fn load_principal(&mut self, _driver: &mut OperationDriver<'_, G>) -> Result<Self::Principal, tonic::Status> {
        let global = &self.global::<G>()?;
        let session = self.session_or_err()?;
        common::get_user_by_id(global, session.user_id).await
    }

    async fn load_resource(&mut self, driver: &mut OperationDriver<'_, G>) -> Result<Self::Resource, tonic::Status> {
        let user_id: UserId = self
            .get_ref()
            .id
            .parse()
            .into_tonic_err_with_field_violation("id", "invalid ID")?;

        let conn = driver.conn().await?;
        common::get_user_by_id_in_tx(conn, user_id).await
    }

    async fn execute(
        self,
        driver: &mut OperationDriver<'_, G>,
        _principal: Self::Principal,
        resource: Self::Resource,
    ) -> Result<Self::Response, tonic::Status> {
        let conn = driver.conn().await?;

        diesel::delete(users::dsl::users)
            .filter(users::dsl::id.eq(resource.id))
            .execute(conn)
            .await
            .into_tonic_internal_err("failed to delete webauthn credential")?;

        Ok(resource.into())
    }
}
