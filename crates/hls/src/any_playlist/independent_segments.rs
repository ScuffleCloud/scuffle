use crate::Tag;

pub struct IndependentSegments;

impl Tag for IndependentSegments {
    const NAME: &'static str = "EXT-X-INDEPENDENT-SEGMENTS";
}
