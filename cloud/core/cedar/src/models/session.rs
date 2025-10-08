use base64::Engine;
use core_db_types::models::{
    MagicLinkRequest, MagicLinkRequestId, UserSession, UserSessionRequest, UserSessionRequestId, UserSessionToken,
    UserSessionTokenId,
};

use crate::macros::cedar_entity;
use crate::{EntityTypeName, entity_type_name};

cedar_entity!(UserSessionRequest, UserSessionRequestId);

cedar_entity!(MagicLinkRequest, MagicLinkRequestId);

impl crate::CedarIdentifiable for UserSession {
    const ENTITY_TYPE: EntityTypeName = entity_type_name!("UserSession");

    fn entity_id(&self) -> cedar_policy::EntityId {
        let user_id = self.user_id.ulid().to_string();
        let fingerprint = base64::prelude::BASE64_STANDARD.encode(&self.device_fingerprint);
        cedar_policy::EntityId::new(format!("{user_id}:{fingerprint}"))
    }
}

impl crate::CedarEntity for UserSession {}

cedar_entity!(UserSessionToken, UserSessionTokenId);
