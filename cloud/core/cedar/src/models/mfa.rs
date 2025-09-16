use core_db_types::models::{MfaTotpCredential, MfaWebauthnCredential};

use crate::macros::cedar_entity;

cedar_entity!(MfaTotpCredential);

cedar_entity!(MfaWebauthnCredential);
