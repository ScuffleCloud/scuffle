use crate::types::ProtoType;

#[derive(Debug, Clone, PartialEq)]
pub enum CelType {
    CelValue,
    Proto(ProtoType),
}
