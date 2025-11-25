use crate::Tag;

#[derive(Debug)]
pub struct Discontinuity;

impl Tag for Discontinuity {
    const NAME: &'static str = "EXT-X-DISCONTINUITY";
}
