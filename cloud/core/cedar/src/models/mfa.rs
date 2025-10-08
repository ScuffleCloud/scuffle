use core_db_types::models::{MfaTotpCredential, MfaTotpCredentialId, MfaWebauthnCredential, MfaWebauthnCredentialId};

use crate::macros::cedar_entity;

cedar_entity!(MfaTotpCredential, MfaTotpCredentialId);

cedar_entity!(MfaWebauthnCredential, MfaWebauthnCredentialId);
