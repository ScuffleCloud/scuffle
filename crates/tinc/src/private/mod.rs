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

#[cfg(feature = "prost")]
mod well_known;
#[cfg(feature = "prost")]
pub use well_known::*;

mod deserializer;
pub use deserializer::*;

mod http;
pub use http::*;
pub use tinc_cel as cel;
mod validation;
pub use validation::*;

mod fmt;
pub use fmt::*;

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

#[inline(always)]
pub fn serde_ser_skip_default<T: Default + PartialEq>(value: &T) -> bool {
    value == &T::default()
}

pub fn deserialize_tracker_target<'de, D, T>(
    state: &mut TrackerSharedState,
    de: D,
    tracker: &mut T,
    target: &mut T::Target,
) -> Result<(), D::Error>
where
    D: serde::Deserializer<'de>,
    T: TrackerDeserializer<'de>,
{
    tinc_cel::CelMode::Serde.set();
    state.in_scope(|| {
        <T as TrackerDeserializer>::deserialize(
            tracker,
            target,
            SerdeDeserializer {
                deserializer: wrapper::DeserializerWrapper::new(de),
            },
        )
    })
}
