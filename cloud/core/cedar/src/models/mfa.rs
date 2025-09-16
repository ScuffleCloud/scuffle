use core_db_types::models::{MfaTotpCredential, MfaWebauthnCredential};

use crate::macros::impl_cedar_identity;

impl_cedar_identity!(MfaTotpCredential);

impl_cedar_identity!(MfaWebauthnCredential);
