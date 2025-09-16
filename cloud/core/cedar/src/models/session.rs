use base64::Engine;
use core_db_types::models::{MagicLinkRequest, UserSession, UserSessionRequest, UserSessionToken};

use crate::macros::impl_cedar_identity;

impl_cedar_identity!(UserSessionRequest);

impl_cedar_identity!(MagicLinkRequest);

impl crate::CedarEntity for UserSession {
    const ENTITY_TYPE: &'static str = "UserSession";

    fn entity_id(&self) -> cedar_policy::EntityId {
        let user_id = (*self.user_id).to_string();
        let fingerprint = base64::prelude::BASE64_STANDARD.encode(&self.device_fingerprint);
        cedar_policy::EntityId::new(format!("{user_id}:{fingerprint}"))
    }
}

impl_cedar_identity!(UserSessionToken);
