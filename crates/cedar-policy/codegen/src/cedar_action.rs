use std::collections::BTreeSet;

use crate::types::{ActionEid, CedarRef, CedarType};

/// Represents a Cedar action with its constraints
#[derive(Debug, Default)]
pub(crate) struct CedarAction {
    pub(crate) principals: Vec<CedarRef>,
    pub(crate) resources: Vec<CedarRef>,
    pub(crate) parents: BTreeSet<ActionEid>,
    pub(crate) context: Option<CedarType>,
}
