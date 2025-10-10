use core_db_types::models::UserSession;
use ext_traits::OptionExt;
use tonic::Code;
use tonic_types::ErrorDetails;

use crate::middleware::ExpiredSession;

pub(crate) trait CoreRequestExt: ext_traits::RequestExt {
    fn session(&self) -> Option<&UserSession> {
        self.extensions().get::<UserSession>()
    }

    fn session_or_err(&self) -> Result<&UserSession, tonic::Status> {
        self.session()
            .into_tonic_err(Code::Unauthenticated, "you must be logged in", ErrorDetails::new())
    }

    fn expired_session_or_err(&self) -> Result<&UserSession, tonic::Status> {
        self.extensions().get::<ExpiredSession>().map(|s| &s.0).into_tonic_err(
            Code::Unauthenticated,
            "you must be logged in",
            ErrorDetails::new(),
        )
    }

    fn dashboard_origin<G: core_traits::ConfigInterface + 'static>(&self) -> Result<url::Url, tonic::Status> {
        self.global::<G>()?
            .dashboard_origin()
            .cloned()
            .or_else(|| self.origin())
            .ok_or_else(|| tonic::Status::invalid_argument("missing origin header and no dashboard origin configured"))
    }
}

impl<T> CoreRequestExt for T where T: ext_traits::RequestExt {}
