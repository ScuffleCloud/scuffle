use std::borrow::Cow;

use bytes::Bytes;
use bytestring::ByteString;

/// A wrapper around a [`Cow`] of a [`Bytes`] object.
#[derive(Debug, Clone, Eq, Hash)]
pub enum BytesCow<'a> {
    /// A borrowed [`Bytes`] object.
    Slice(&'a [u8]),
    /// A static [`Bytes`] object.
    StaticSlice(&'static [u8]),
    /// An owned Vec of bytes.
    Vec(Vec<u8>),
    /// An owned [`Bytes`] object.
    Bytes(Bytes),
}

impl Default for BytesCow<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> BytesCow<'a> {
    /// Creates an empty [`BytesCow`] object.
    pub fn new() -> Self {
        Self::from_static(b"")
    }

    /// Creates a new [`BytesCow`] from a static slice.
    pub fn from_static(slice: &'static [u8]) -> Self {
        Self::StaticSlice(slice)
    }

    /// Creates a new [`BytesCow`] from a slice of bytes.
    pub fn from_slice(slice: &'a [u8]) -> Self {
        Self::Slice(slice)
    }

    /// Creates a new [`BytesCow`] from a [`Bytes`] object.
    pub fn from_bytes(bytes: Bytes) -> Self {
        Self::Bytes(bytes)
    }

    /// Creates a new [`BytesCow`] from a [`Cow`] of a [`Bytes`] object.
    pub fn from_cow(cow: Cow<'a, [u8]>) -> Self {
        match cow {
            Cow::Borrowed(slice) => Self::Slice(slice),
            Cow::Owned(bytes) => Self::Vec(bytes),
        }
    }

    /// Creates a new [`BytesCow`] from a [`Vec`] of bytes.
    pub fn from_vec(bytes: Vec<u8>) -> Self {
        Self::Vec(bytes)
    }

    /// Converts the object into a [`Bytes`] object.
    pub fn into_bytes(self) -> Bytes {
        match self {
            Self::Slice(slice) => Bytes::copy_from_slice(slice),
            Self::StaticSlice(slice) => Bytes::from_static(slice),
            Self::Vec(bytes) => Bytes::from(bytes),
            Self::Bytes(bytes) => bytes,
        }
    }

    /// Returns a reference to the inner data as a slice.
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Slice(slice) => slice,
            Self::StaticSlice(slice) => slice,
            Self::Vec(bytes) => bytes.as_slice(),
            Self::Bytes(bytes) => bytes.as_ref(),
        }
    }
}

impl AsRef<[u8]> for BytesCow<'_> {
    fn as_ref(&self) -> &[u8] {
        match self {
            BytesCow::Slice(slice) => slice,
            BytesCow::StaticSlice(slice) => slice,
            BytesCow::Vec(bytes) => bytes.as_slice(),
            BytesCow::Bytes(bytes) => bytes.as_ref(),
        }
    }
}

impl<'a> From<Cow<'a, [u8]>> for BytesCow<'a> {
    fn from(cow: Cow<'a, [u8]>) -> Self {
        BytesCow::from_cow(cow)
    }
}

impl From<Bytes> for BytesCow<'_> {
    fn from(bytes: Bytes) -> Self {
        BytesCow::from_bytes(bytes)
    }
}

impl<'a> From<&'a [u8]> for BytesCow<'a> {
    fn from(bytes: &'a [u8]) -> Self {
        BytesCow::from_slice(bytes)
    }
}

impl From<Vec<u8>> for BytesCow<'_> {
    fn from(bytes: Vec<u8>) -> Self {
        BytesCow::from_vec(bytes)
    }
}

impl<T> PartialEq<T> for BytesCow<'_>
where
    T: AsRef<[u8]>,
{
    fn eq(&self, other: &T) -> bool {
        self.as_bytes() == other.as_ref()
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for BytesCow<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(self.as_bytes())
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for BytesCow<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct BytesCowVisitor;

        impl<'de> serde::de::Visitor<'de> for BytesCowVisitor {
            type Value = BytesCow<'de>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a byte slice")
            }

            fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(BytesCow::from_vec(v))
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(BytesCow::from_vec(v.to_vec()))
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(BytesCow::from_vec(v.as_bytes().to_vec()))
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(BytesCow::from_vec(v.into_bytes()))
            }

            fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
                where
                E: serde::de::Error,
            {
                Ok(BytesCow::from_slice(v))
            }

            fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(BytesCow::from_slice(v.as_bytes()))
            }
        }

        deserializer.deserialize_bytes(BytesCowVisitor)
    }
}

/// A helper wrapper around a [`Cow`] of a [`ByteString`] object.
#[derive(Debug, Clone, Eq, Hash, PartialOrd, Ord)]
pub enum StringCow<'a> {
    /// A borrowed [`ByteString`] object.
    Ref(&'a str),
    /// A static borrowed [`ByteString`] object.
    StaticRef(&'static str),
    /// An owned [`String`] object.
    String(String),
    /// An owned [`ByteString`] object.
    Bytes(ByteString),
}

impl Default for StringCow<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> StringCow<'a> {
    /// Creates an empty [`StringCow`] object.
    pub fn new() -> Self {
        Self::from_static("")
    }

    /// Creates a new [`StringCow`] from a static slice.
    pub fn from_static(slice: &'static str) -> Self {
        StringCow::StaticRef(slice)
    }

    /// Creates a new [`StringCow`] from a [`ByteString`] object.
    pub fn from_bytes(bytes: ByteString) -> Self {
        StringCow::Bytes(bytes)
    }

    /// Creates a new [`StringCow`] from a [`Cow`] of a [`str`] object.
    pub fn from_cow(cow: Cow<'a, str>) -> Self {
        match cow {
            Cow::Borrowed(slice) => StringCow::Ref(slice),
            Cow::Owned(string) => StringCow::String(string),
        }
    }

    /// Creates a new [`StringCow`] from a static slice.
    pub fn from_ref(slice: &'a str) -> Self {
        StringCow::Ref(slice)
    }

    /// Creates a new [`StringCow`] from a [`String`] object.
    pub fn from_string(string: String) -> Self {
        StringCow::String(string)
    }

    /// Converts the object into a [`ByteString`] object.
    pub fn into_bytes(self) -> ByteString {
        match self {
            StringCow::Ref(slice) => ByteString::from(slice),
            StringCow::StaticRef(slice) => ByteString::from_static(slice),
            StringCow::String(string) => ByteString::from(string),
            StringCow::Bytes(bytes) => bytes,
        }
    }

    /// Returns a reference to the inner data as a slice.
    pub fn as_str(&self) -> &str {
        match self {
            StringCow::Ref(slice) => slice,
            StringCow::StaticRef(slice) => slice,
            StringCow::String(string) => string.as_str(),
            StringCow::Bytes(bytes) => bytes.as_ref(),
        }
    }
}

impl AsRef<str> for StringCow<'_> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<'a> From<Cow<'a, str>> for StringCow<'a> {
    fn from(cow: Cow<'a, str>) -> Self {
        StringCow::from_cow(cow)
    }
}

impl<'a> From<&'a str> for StringCow<'a> {
    fn from(slice: &'a str) -> Self {
        StringCow::from_ref(slice)
    }
}

impl From<String> for StringCow<'_> {
    fn from(string: String) -> Self {
        StringCow::from_string(string)
    }
}

impl From<ByteString> for StringCow<'_> {
    fn from(bytes: ByteString) -> Self {
        StringCow::from_bytes(bytes)
    }
}

impl<T> PartialEq<T> for StringCow<'_>
where
    T: AsRef<str>,
{
    fn eq(&self, other: &T) -> bool {
        self.as_str() == other.as_ref()
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for StringCow<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for StringCow<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct StringCowVisitor;

        impl<'de> serde::de::Visitor<'de> for StringCowVisitor {
            type Value = StringCow<'de>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string")
            }

            fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(StringCow::from_ref(v))
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(StringCow::from_string(v))
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                where
                E: serde::de::Error,
            {
                Ok(StringCow::from_string(v.to_string()))
            }
        }

        deserializer.deserialize_any(StringCowVisitor)
    }
}
