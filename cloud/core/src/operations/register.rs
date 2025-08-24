use base64::Engine;
use diesel::{BoolExpressionMethods, ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use tonic::Code;
use tonic_types::{ErrorDetails, StatusExt};

use crate::cedar::{Action, CoreApplication, Unauthenticated};
use crate::http_ext::RequestExt;
use crate::models::{EmailRegistrationRequest, EmailRegistrationRequestId, User, UserId};
use crate::operations::Operation;
use crate::schema::{email_registration_requests, user_emails};
use crate::std_ext::{OptionExt, ResultExt};
use crate::{CoreConfig, captcha, common, emails};

impl<G: CoreConfig> Operation<G> for tonic::Request<pb::scufflecloud::core::v1::RegisterWithEmailRequest> {
    type Principal = Unauthenticated;
    type Resource = CoreApplication;
    type Response = ();

    const ACTION: Action = Action::RegisterWithEmail;

    async fn validate(&mut self) -> Result<(), tonic::Status> {
        let global = &self.global::<G>()?;

        // Check captcha
        let captcha = self.get_ref().captcha.clone().require("captcha")?;
        match captcha.provider() {
            pb::scufflecloud::core::v1::CaptchaProvider::Turnstile => {
                captcha::turnstile::verify_in_tonic(global, &captcha.token).await?;
            }
        }

        Ok(())
    }

    async fn load_principal(&mut self, _tx: &mut diesel_async::AsyncPgConnection) -> Result<Self::Principal, tonic::Status> {
        Ok(Unauthenticated)
    }

    async fn load_resource(&mut self, _tx: &mut diesel_async::AsyncPgConnection) -> Result<Self::Resource, tonic::Status> {
        Ok(CoreApplication)
    }

    async fn execute(
        self,
        tx: &mut diesel_async::AsyncPgConnection,
        _principal: Self::Principal,
        _resource: Self::Resource,
    ) -> Result<Self::Response, tonic::Status> {
        let global = &self.global::<G>()?;
        let payload = self.into_inner();

        let email = common::normalize_email(&payload.email);

        // Generate random code
        let code = common::generate_random_bytes().into_tonic_internal_err("failed to generate registration code")?;
        let code_base64 = base64::prelude::BASE64_URL_SAFE.encode(code);

        // Check if email is already registered
        if user_emails::dsl::user_emails
            .find(&email)
            .select(user_emails::dsl::email)
            .first::<String>(tx)
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

        // Create email registration request
        let registration_request = EmailRegistrationRequest {
            id: EmailRegistrationRequestId::new(),
            user_id: None,
            email: email.clone(),
            code: code.to_vec(),
            expires_at: chrono::Utc::now() + global.email_registration_request_timeout(),
        };

        diesel::insert_into(email_registration_requests::dsl::email_registration_requests)
            .values(registration_request)
            .execute(tx)
            .await
            .into_tonic_internal_err("failed to insert email registration request")?;

        // Send email
        let email = emails::register_with_email_email(global, email, code_base64)
            .await
            .into_tonic_internal_err("failed to render registration email")?;
        global
            .email_service()
            .send_email(email)
            .await
            .into_tonic_internal_err("failed to send registration email")?;

        Ok(())
    }
}

impl<G: CoreConfig> Operation<G> for tonic::Request<pb::scufflecloud::core::v1::CompleteRegisterWithEmailRequest> {
    type Principal = User;
    type Resource = CoreApplication;
    type Response = pb::scufflecloud::core::v1::NewUserSessionToken;

    const ACTION: Action = Action::CompleteRegisterWithEmail;

    async fn load_principal(&mut self, tx: &mut diesel_async::AsyncPgConnection) -> Result<Self::Principal, tonic::Status> {
        // Delete email registration request
        let Some(registration_request) = diesel::delete(email_registration_requests::dsl::email_registration_requests)
            .filter(
                email_registration_requests::dsl::code
                    .eq(&self.get_ref().code)
                    .and(email_registration_requests::dsl::user_id.is_null())
                    .and(email_registration_requests::dsl::expires_at.gt(chrono::Utc::now())),
            )
            .returning(EmailRegistrationRequest::as_select())
            .get_result::<EmailRegistrationRequest>(tx)
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

        Ok(User {
            id: UserId::new(),
            preferred_name: None,
            first_name: None,
            last_name: None,
            password_hash: None,
            primary_email: Some(registration_request.email),
        })
    }

    async fn load_resource(&mut self, _tx: &mut diesel_async::AsyncPgConnection) -> Result<Self::Resource, tonic::Status> {
        Ok(CoreApplication)
    }

    async fn execute(
        self,
        tx: &mut diesel_async::AsyncPgConnection,
        principal: Self::Principal,
        _resource: Self::Resource,
    ) -> Result<Self::Response, tonic::Status> {
        let global = &self.global::<G>()?;
        let ip_info = self.ip_address_info()?;

        let device = self.into_inner().device.require("device")?;
        common::create_user(tx, &principal).await?;
        let new_token = common::create_session(global, tx, principal.id, device, &ip_info, false).await?;

        Ok(new_token)
    }
}
