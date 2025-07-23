use crate::http_ext::RequestExt;
use crate::services::CoreSvc;
use crate::std_ext::{OptionExt, ResultExt};
use crate::{CoreConfig, captcha, webauthn};

#[async_trait::async_trait]
impl<G: CoreConfig> pb::scufflecloud::core::v1::webauthn_challenges_service_server::WebauthnChallengesService
    for CoreSvc<G>
{
    async fn create_webauthn_challenge(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::CreateWebauthnChallengeRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::CreateWebauthnChallengeResponse>, tonic::Status> {
        let global = &req.global::<G>()?;
        let payload = req.into_inner();

        // Check captcha
        let captcha = payload.captcha.require("captcha")?;
        match captcha.provider() {
            pb::scufflecloud::core::v1::CaptchaProvider::Turnstile => {
                captcha::turnstile::verify_in_tonic(global, &captcha.token).await?;
            }
        }

        let challenge = webauthn::new_challenge(global).into_tonic_internal_err("failed to create webauthn challenge")?;

        Ok(tonic::Response::new(
            pb::scufflecloud::core::v1::CreateWebauthnChallengeResponse { challenge },
        ))
    }
}
