use core_db_types::models::{Sha256, UserId};

pub(crate) trait AuthSessionExt {
    async fn auth_session<G: core_traits::Global>(&self) -> tonic::Result<Option<AuthSession>>;
}

pub(crate) enum AuthSession {
    User(UserAuthSession),
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
}

impl<T> AuthSessionExt for tonic::Request<T> {
    async fn auth_session<G: core_traits::Global>(&self) -> tonic::Result<Option<AuthSession>> {
        todo!("not implemented")
    }
}
