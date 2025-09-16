use base64::Engine;
use core_db_types::models::{MagicLinkRequest, UserSession, UserSessionRequest, UserSessionToken};

use crate::macros::cedar_entity;

cedar_entity!(UserSessionRequest);

cedar_entity!(MagicLinkRequest);

impl crate::CedarIdentifiable for UserSession {
    const ENTITY_TYPE: &'static str = "UserSession";

    fn entity_id(&self) -> cedar_policy::EntityId {
        let user_id = (*self.user_id).to_string();
        let fingerprint = base64::prelude::BASE64_STANDARD.encode(&self.device_fingerprint);
        cedar_policy::EntityId::new(format!("{user_id}:{fingerprint}"))
    }
}

impl crate::CedarEntity for UserSession {}

cedar_entity!(UserSessionToken);
