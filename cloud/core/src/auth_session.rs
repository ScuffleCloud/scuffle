use std::collections::HashMap;

use core_db_types::models::{Sha256, UserId};
use tonic_types::{ErrorDetails, StatusExt};

pub(crate) trait AuthSessionExt {
    async fn auth_session<G: core_traits::Global>(&self) -> tonic::Result<Option<AuthSession>>;
    async fn auth_user<G: core_traits::Global>(&self) -> tonic::Result<Option<UserAuthSession>> {
        let Some(auth_session) = self.auth_session::<G>().await? else {
            return Ok(None);
        };

        auth_session.user().map(Some)
    }
}

pub(crate) trait AuthSessionResultExt: Sized {
    type Output;
    fn required(self) -> tonic::Result<Self::Output>;
}

impl AuthSessionResultExt for tonic::Result<Option<AuthSession>> {
    type Output = AuthSession;

    fn required(self) -> tonic::Result<Self::Output> {
        match self {
            Ok(s) => s.required(),
            Err(e) => Err(e),
        }
    }
}

impl AuthSessionResultExt for tonic::Result<AuthSession> {
    type Output = AuthSession;

    fn required(self) -> tonic::Result<Self::Output> {
        match self {
            Ok(session) => Ok(session),
            Err(e) => Err(e),
        }
    }
}

impl AuthSessionResultExt for tonic::Result<Option<UserAuthSession>> {
    type Output = UserAuthSession;

    fn required(self) -> tonic::Result<Self::Output> {
        match self {
            Ok(s) => s.required(),
            Err(e) => Err(e),
        }
    }
}

impl AuthSessionResultExt for tonic::Result<UserAuthSession> {
    type Output = UserAuthSession;

    fn required(self) -> tonic::Result<Self::Output> {
        match self {
            Ok(session) => Ok(session),
            Err(e) => Err(e),
        }
    }
}

impl AuthSessionResultExt for Option<AuthSession> {
    type Output = AuthSession;

    fn required(self) -> tonic::Result<Self::Output> {
        match self {
            Some(session) => Ok(session),
            None => Err(tonic::Status::with_error_details(
                tonic::Code::Unauthenticated,
                "authentication required",
                ErrorDetails::new(),
            )),
        }
    }
}

impl AuthSessionResultExt for Option<UserAuthSession> {
    type Output = UserAuthSession;

    fn required(self) -> tonic::Result<Self::Output> {
        match self {
            Some(session) => Ok(session),
            None => Err(tonic::Status::with_error_details(
                tonic::Code::Unauthenticated,
                "user authentication required",
                ErrorDetails::new(),
            )),
        }
    }
}

#[non_exhaustive]
pub(crate) enum AuthSession {
    User(UserAuthSession),
}

impl AuthSession {
    pub(crate) fn user(self) -> tonic::Result<UserAuthSession> {
        match self {
            AuthSession::User(session) => Ok(session),
            #[allow(unreachable_patterns)]
            _ => Err(tonic::Status::with_error_details(
                tonic::Code::Unauthenticated,
                "only users can access this resource",
                ErrorDetails::with_error_info("this resource is only accessible to users", "auth", HashMap::new()),
            )),
        }
    }
}

pub(crate) struct UserAuthSession {
    id: UserId,
    device_fingerprint: Sha256,
    has_mfa: bool,
}

impl UserAuthSession {
    pub(crate) fn id(&self) -> UserId {
        self.id
    }

    pub(crate) fn device_fingerprint(&self) -> Sha256 {
        self.device_fingerprint
    }

    pub(crate) fn has_mfa(&self) -> bool {
        self.has_mfa
    }

    pub(crate) fn with_mfa_required(self) -> tonic::Result<Self> {
        self.mfa_required()?;
        Ok(self)
    }

    pub(crate) fn mfa_required(&self) -> tonic::Result<()> {
        if !self.has_mfa {
            return Err(tonic::Status::with_error_details(
                tonic::Code::PermissionDenied,
                "multi-factor authentication required",
                ErrorDetails::with_error_info(
                    "multi-factor authentication required",
                    "auth",
                    HashMap::from_iter([("requires_mfa".into(), "true".into())]),
                ),
            ));
        }
        Ok(())
    }
}

impl<T> AuthSessionExt for tonic::Request<T> {
    async fn auth_session<G: core_traits::Global>(&self) -> tonic::Result<Option<AuthSession>> {
        todo!("not implemented")
    }
}
