use crate::Tag;

pub struct EndList;

impl Tag for EndList {
    const NAME: &'static str = "EXT-X-ENDLIST";
}
