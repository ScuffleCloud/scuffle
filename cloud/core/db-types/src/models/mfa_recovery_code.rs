use std::borrow::Cow;

use crate::id::{MfaRecoveryCodeId, UserId};

#[derive(Debug, Clone, diesel::Queryable, diesel::Selectable)]
#[diesel(table_name = crate::schema::mfa_recovery_codes)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MfaRecoveryCode {
    pub id: MfaRecoveryCodeId,
    pub user_id: UserId,
    pub code_hash: String,
}

#[derive(Debug, Clone, diesel::Insertable, diesel::Identifiable, diesel::AsChangeset, bon::Builder)]
#[diesel(table_name = crate::schema::mfa_recovery_codes)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewMfaRecoveryCode<'a> {
    #[builder(default)]
    pub id: MfaRecoveryCodeId,
    pub user_id: UserId,
    pub code_hash: Cow<'a, str>,
}
