pub mod reexports {
    pub use {chrono, serde, tinc_pb};
}

pub mod serde_helpers;

pub struct Schema {}

pub trait HasSchema<K> {
    fn schema() -> Schema;
}
