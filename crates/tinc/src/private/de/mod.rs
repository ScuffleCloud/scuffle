use std::borrow::Cow;

use tracker::Tracker;
use tracker::struct_::{StructDeserializer, TrackerStruct};
pub use wrapper::DeserializerWrapper;

pub mod bit_field;
pub mod map;
pub mod tracker;
pub mod wrapper;

pub trait StructField: Sized {
    fn idx(&self) -> usize;
    fn name(&self) -> &'static str;
    fn from_str(s: &str) -> Option<Self>;
}

pub struct StructIdentifierDeserializer<F>(std::marker::PhantomData<F>);

impl<F> Default for StructIdentifierDeserializer<F> {
    fn default() -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<F: StructField> StructIdentifierDeserializer<F> {
    pub const fn new() -> Self {
        Self(std::marker::PhantomData)
    }
}

pub enum StructIdentifier<'a, F> {
    Field(F),
    Unknown(Cow<'a, str>),
}

impl<'a, F: StructField> serde::de::Visitor<'a> for StructIdentifierDeserializer<F> {
    type Value = StructIdentifier<'a, F>;

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(F::from_str(v).map_or_else(
            || StructIdentifier::Unknown(v.to_owned().into()),
            |field| StructIdentifier::Field(field),
        ))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(F::from_str(&v).map_or_else(|| StructIdentifier::Unknown(v.into()), |field| StructIdentifier::Field(field)))
    }

    fn visit_borrowed_str<E>(self, v: &'a str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(F::from_str(v).map_or_else(
            || StructIdentifier::Unknown(v.to_owned().into()),
            |field| StructIdentifier::Field(field),
        ))
    }

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a valid field name")
    }
}

impl<'de, F> serde::de::DeserializeSeed<'de> for StructIdentifierDeserializer<F>
where
    F: StructField,
{
    type Value = StructIdentifier<'de, F>;

    #[inline]
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_identifier(self)
    }
}

pub trait TrackedStructDeserializer<'de>: Sized {
    const NAME: &'static str;
    const FIELDS: &'static [&'static str];

    type Field: StructField;

    fn deserialize<D>(
        &mut self,
        field: Self::Field,
        tracker: Tracker<'_, TrackerStruct>,
        deserializer: D,
    ) -> Result<(), D::Error>
    where
        D: DeserializeFieldValue<'de>;

    fn handle_unknown_field<E>(&mut self, field: &str, tracker: Tracker<'_, TrackerStruct>) -> Result<(), E>
    where
        E: serde::de::Error,
    {
        // todo: handle unknown fields
        let _ = field;
        let _ = tracker;
        Ok(())
    }

    fn verify_deserialize<E>(&self, tracker: Tracker<'_, TrackerStruct>) -> Result<(), E>
    where
        E: serde::de::Error,
    {
        let _ = tracker;
        Ok(())
    }
}

impl<'de, T> TrackedStructDeserializer<'de> for Box<T>
where
    T: TrackedStructDeserializer<'de>,
{
    type Field = T::Field;

    const FIELDS: &'static [&'static str] = T::FIELDS;
    const NAME: &'static str = T::NAME;

    #[inline(always)]
    fn deserialize<D>(
        &mut self,
        field: Self::Field,
        tracker: Tracker<'_, TrackerStruct>,
        deserializer: D,
    ) -> Result<(), D::Error>
    where
        D: DeserializeFieldValue<'de>,
    {
        T::deserialize(self.as_mut(), field, tracker, deserializer)
    }

    #[inline(always)]
    fn handle_unknown_field<E>(&mut self, field: &str, tracker: Tracker<'_, TrackerStruct>) -> Result<(), E>
    where
        E: serde::de::Error,
    {
        T::handle_unknown_field(self.as_mut(), field, tracker)
    }

    #[inline(always)]
    fn verify_deserialize<E>(&self, tracker: Tracker<'_, TrackerStruct>) -> Result<(), E>
    where
        E: serde::de::Error,
    {
        T::verify_deserialize(self, tracker)
    }
}

impl<'de, T: TrackedStructDeserializer<'de>> serde::de::DeserializeSeed<'de> for StructDeserializer<'_, T> {
    type Value = ();

    fn deserialize<D>(mut self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_struct(T::NAME, T::FIELDS, &mut self)
    }
}

pub trait DeserializeFieldValue<'de> {
    type Error: serde::de::Error;

    fn deserialize<T>(self) -> Result<T, Self::Error>
    where
        T: serde::de::Deserialize<'de>;

    fn deserialize_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>;
}

#[macro_export]
#[doc(hidden)]
macro_rules! __tinc_field_from_str {
    (
        $s:expr,
        $($literal:literal => $expr:expr),*$(,)?
    ) => {
        match $s {
            $($literal => Some($expr),)*
            _ => None,
        }
    };
}
