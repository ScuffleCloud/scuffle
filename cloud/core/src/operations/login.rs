use tonic_types::{ErrorDetails, StatusExt};

use crate::cedar::{Action, CoreApplication};
use crate::http_ext::RequestExt;
use crate::models::User;
use crate::operations::Operation;
use crate::std_ext::OptionExt;
use crate::{CoreConfig, captcha, common};

impl<G: CoreConfig> Operation<G> for tonic::Request<pb::scufflecloud::core::v1::LoginWithEmailAndPasswordRequest> {
    type Principal = User;
    type Resource = CoreApplication;
    type Response = pb::scufflecloud::core::v1::NewUserSessionToken;

    const ACTION: Action = Action::LoginWithEmailPassword;

    async fn validate(&self) -> Result<(), tonic::Status> {
        let global = &self.global::<G>()?;
        let captcha = self.get_ref().captcha.clone().require("captcha")?;

        // Check captcha
        match captcha.provider() {
            pb::scufflecloud::core::v1::CaptchaProvider::Turnstile => {
                captcha::turnstile::verify_in_tonic(global, &captcha.token).await?;
            }
        }

        Ok(())
    }

    async fn load_principal(&self, tx: &mut diesel_async::AsyncPgConnection) -> Result<Self::Principal, tonic::Status> {
        common::get_user_by_email(tx, &self.get_ref().email).await
    }

    async fn load_resource(&self, _tx: &mut diesel_async::AsyncPgConnection) -> Result<Self::Resource, tonic::Status> {
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
        let payload = self.into_inner();

        let device = payload.device.require("device")?;

        // Verify password
        let Some(password_hash) = &principal.password_hash else {
            return Err(tonic::Status::with_error_details(
                tonic::Code::FailedPrecondition,
                "user does not have a password set",
                ErrorDetails::new(),
            ));
        };

        common::verify_password(password_hash, &payload.password)?;

        common::create_session(global, tx, principal.id, device, &ip_info, true).await
    }
}
