pub mod buffer;
pub mod const_macros;
pub mod wrapper;

pub use tinc_derive::Tracker;

mod oneof;
pub use oneof::*;

mod error;
pub use error::*;

mod tracker;
pub use tracker::*;

mod identifier;
pub use identifier::*;

mod primitive;
pub use primitive::*;

mod map;
pub use map::*;

mod optional;
pub use optional::*;

mod enum_;
pub use enum_::*;

mod struct_;
pub use struct_::*;

mod repeated;
pub use repeated::*;

mod expected;
pub use expected::*;

mod well_known;
pub use well_known::*;

mod deserializer;
pub use deserializer::*;

mod http;
pub use http::*;

#[macro_export]
#[doc(hidden)]
macro_rules! __tinc_field_from_str {
    (
        $s:expr,
        $($literal:literal => $expr:expr),*
        $(,flattened: [$($ident:ident),*$(,)?])?
        $(,)?
    ) => {
        match $s {
            $($literal => Ok($expr),)*
            _ => {
                $($(
                    if let Ok(result) = ::core::str::FromStr::from_str($s) {
                        return Ok(Self::$ident(result));
                    }
                )*)?

                Err(())
            },
        }
    };
}

#[inline(always)]
pub fn tracker_allow_duplicates<T: Tracker>(tracker: Option<&T>) -> bool {
    tracker.is_none_or(|tracker| tracker.allow_duplicates())
}

pub fn deserialize_tracker_target<'de, D, T>(de: D, target: &mut T::Target, tracker: &mut T) -> Result<(), D::Error>
where
    D: serde::Deserializer<'de>,
    T: TrackerDeserializer<'de>,
{
    <T as TrackerDeserializer>::deserialize(
        tracker,
        target,
        SerdeDeserializer {
            deserializer: wrapper::DeserializerWrapper::new(de),
        },
    )
}
