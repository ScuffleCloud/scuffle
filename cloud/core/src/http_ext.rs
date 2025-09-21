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
}

impl<T> CoreRequestExt for T where T: ext_traits::RequestExt {}
