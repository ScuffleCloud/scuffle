use crate::types::ProtoType;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum CelType {
    CelValue,
    Proto(ProtoType),
}
