use std::borrow::Cow;
use std::str::FromStr;

pub trait Identifier: FromStr + Copy + Eq + PartialEq + Ord + std::hash::Hash + PartialOrd {
    const OPTIONS: &'static [&'static str];

    fn name(&self) -> &'static str;
}

pub trait IdentifierFor {
    const NAME: &'static str;

    type Identifier: Identifier;
}

pub struct IdentifierDeserializer<F>(std::marker::PhantomData<F>);

impl<F> Default for IdentifierDeserializer<F> {
    fn default() -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<F> IdentifierDeserializer<F> {
    pub const fn new() -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<F: Identifier> IdentifierDeserializer<F> {
    fn visit_owned_borrowed_or_ref<'de>(self, v: OwnedBorrowedOrRef<'de, '_>) -> IdentifiedValue<'de, F> {
        F::from_str(v.as_ref()).map_or_else(
            |_| IdentifiedValue::Unknown(v.into_cow()),
            |field| IdentifiedValue::Found(field),
        )
    }
}

impl<'a, F: Identifier> serde::de::Visitor<'a> for IdentifierDeserializer<F> {
    type Value = IdentifiedValue<'a, F>;

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(self.visit_owned_borrowed_or_ref(OwnedBorrowedOrRef::Ref(v)))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(self.visit_owned_borrowed_or_ref(OwnedBorrowedOrRef::Owned(v)))
    }

    fn visit_borrowed_str<E>(self, v: &'a str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(self.visit_owned_borrowed_or_ref(OwnedBorrowedOrRef::Borrowed(v)))
    }

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a field name")
    }
}

impl<'de, F> serde::de::DeserializeSeed<'de> for IdentifierDeserializer<F>
where
    F: Identifier,
{
    type Value = IdentifiedValue<'de, F>;

    #[inline]
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_identifier(self)
    }
}

pub enum IdentifiedValue<'a, F> {
    Found(F),
    Unknown(Cow<'a, str>),
}

enum OwnedBorrowedOrRef<'de, 'a> {
    Owned(String),
    Borrowed(&'de str),
    Ref(&'a str),
}

impl AsRef<str> for OwnedBorrowedOrRef<'_, '_> {
    fn as_ref(&self) -> &str {
        match self {
            Self::Owned(s) => s.as_str(),
            Self::Borrowed(s) => s,
            Self::Ref(s) => s,
        }
    }
}

impl<'de> OwnedBorrowedOrRef<'de, '_> {
    fn into_cow(self) -> Cow<'de, str> {
        match self {
            Self::Owned(s) => Cow::Owned(s),
            Self::Borrowed(s) => Cow::Borrowed(s),
            Self::Ref(s) => Cow::Owned(s.to_string()),
        }
    }
}

impl<T: IdentifierFor> IdentifierFor for Option<T> {
    type Identifier = T::Identifier;

    const NAME: &'static str = T::NAME;
}

impl<T: IdentifierFor> IdentifierFor for Box<T> {
    type Identifier = T::Identifier;

    const NAME: &'static str = T::NAME;
}
