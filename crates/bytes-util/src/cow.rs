use std::{borrow::Cow, hash::Hash};

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

    /// Converts the object into an owned [`BytesCow`] object.
    pub fn to_owned(self) -> BytesCow<'static> {
        match self {
            Self::Slice(slice) => BytesCow::from_vec(slice.to_vec()),
            Self::StaticSlice(slice) => BytesCow::from_static(slice),
            Self::Vec(bytes) => BytesCow::from_vec(bytes),
            Self::Bytes(bytes) => BytesCow::from_bytes(bytes),
        }
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

    /// Returns a reference to the inner data as a [`&str`]
    pub fn as_str(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(self.as_bytes())
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

    /// Returns a reference to the inner data as a [`ByteString`] object.
    ///
    /// Returns back [`BytesCow`] in the [`Err`] variant if the data is not valid utf-8.
    pub fn into_string_cow(self) -> Result<StringCow<'a>, Self> {
        if std::str::from_utf8(self.as_bytes()).is_ok() {
            match self {
                // Safety: We check that the string is valid utf-8 above.
                Self::Slice(slice) => Ok(StringCow::from_ref(unsafe { std::str::from_utf8_unchecked(slice) })),
                // Safety: We check that the string is valid utf-8 above.
                Self::StaticSlice(slice) => Ok(StringCow::from_static(unsafe { std::str::from_utf8_unchecked(slice) })),
                // Safety: We check that the string is valid utf-8 above.
                Self::Vec(bytes) => Ok(StringCow::from_string(unsafe { String::from_utf8_unchecked(bytes) })),
                // Safety: We check that the string is valid utf-8 above.
                Self::Bytes(bytes) => Ok(StringCow::from_bytes(unsafe { ByteString::from_bytes_unchecked(bytes) })),
            }
        } else {
            Err(self)
        }
    }

    /// Handles a visitor for the [`BytesCow`] object.
    #[cfg(feature = "serde")]
    pub fn handle_visitor<V, E>(self, visitor: V) -> Result<V::Value, E>
    where
        V: serde::de::Visitor<'a>,
        E: serde::de::Error,
    {
        match self {
            Self::Slice(slice) => visitor.visit_borrowed_bytes(slice),
            Self::StaticSlice(slice) => visitor.visit_borrowed_bytes(slice),
            Self::Vec(bytes) => visitor.visit_byte_buf(bytes),
            Self::Bytes(bytes) => visitor.visit_bytes(bytes.as_ref()),
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

impl std::ops::Deref for BytesCow<'_> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_bytes()
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

/// A serde visitor for [`BytesCow`] objects.
#[cfg(feature = "serde")]
pub struct BytesCowVisitor<'de>(Option<BytesCow<'de>>);

impl Default for BytesCowVisitor<'_> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "serde")]
impl<'de> BytesCowVisitor<'de> {
    /// The name of the newtype struct for [`BytesCow`] objects.
    /// This can be used to check if a visitor is a [`BytesCowVisitor`]
    /// you can then cast the visitor to a [`BytesCowVisitor`] and call [`BytesCowVisitor::set`]
    /// to set the value of the visitor.
    /// You can then call [`serde::de::Visitor::visit_unit`] to get the value of the visitor.
    pub const NEW_TYPE_NAME: &'static str = concat!("__internal__BytesCowSerde_new_type_name-", env!("CARGO_PKG_VERSION"));

    /// Creates a new [`BytesCowVisitor`].
    pub fn new() -> Self {
        Self(None)
    }

    /// Sets the value of the visitor.
    pub fn set(&mut self, value: BytesCow<'de>) {
        self.0 = Some(value);
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::de::Visitor<'de> for BytesCowVisitor<'de> {
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

    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if let Some(value) = self.0 {
            Ok(value)
        } else {
            deserializer.deserialize_bytes(self)
        }
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for BytesCow<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_newtype_struct(BytesCowVisitor::NEW_TYPE_NAME, BytesCowVisitor::new())
    }
}

/// A helper wrapper around a [`Cow`] of a [`ByteString`] object.
#[derive(Debug, Clone, Eq)]
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

impl Hash for StringCow<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

impl PartialOrd for StringCow<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
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

    /// Converts the object into an owned [`StringCow`] object.
    pub fn to_owned(self) -> StringCow<'static> {
        match self {
            Self::Ref(slice) => StringCow::from_string(slice.to_owned()),
            Self::StaticRef(slice) => StringCow::from_static(slice),
            Self::String(string) => StringCow::from_string(string),
            Self::Bytes(bytes) => StringCow::from_bytes(bytes),
        }
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
    pub fn into_bytesring(self) -> ByteString {
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

    /// Handles a visitor for the [`StringCow`] object.
    #[cfg(feature = "serde")]
    pub fn handle_visitor<V, E>(self, visitor: V) -> Result<V::Value, E>
    where
        V: serde::de::Visitor<'a>,
        E: serde::de::Error,
    {
        match self {
            Self::Ref(slice) => visitor.visit_borrowed_str(slice),
            Self::StaticRef(slice) => visitor.visit_borrowed_str(slice),
            Self::String(string) => visitor.visit_string(string),
            Self::Bytes(bytes) => visitor.visit_bytes(bytes.as_ref()),
        }
    }

    /// Converts the object into a [`BytesCow`] object.
    pub fn into_bytes_cow(self) -> BytesCow<'a> {
        match self {
            Self::Ref(slice) => BytesCow::from_slice(slice.as_bytes()),
            Self::StaticRef(slice) => BytesCow::from_static(slice.as_bytes()),
            Self::String(string) => BytesCow::from_vec(string.into_bytes()),
            Self::Bytes(bytes) => BytesCow::from_bytes(bytes.into_bytes()),
        }
    }
}

impl AsRef<str> for StringCow<'_> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::ops::Deref for StringCow<'_> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
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

/// A serde visitor for [`StringCow`] objects.
#[cfg(feature = "serde")]
pub struct StringCowVisitor<'de>(Option<StringCow<'de>>);

impl Default for StringCowVisitor<'_> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "serde")]
impl<'de> StringCowVisitor<'de> {
    /// The name of the newtype struct for [`StringCow`] objects.
    /// This can be used to check if a visitor is a [`StringCowVisitor`]
    /// you can then cast the visitor to a [`StringCowVisitor`] and call [`StringCowVisitor::set`]
    /// to set the value of the visitor.
    pub const NEW_TYPE_NAME: &'static str = concat!("__internal__StringCowSerde_new_type_name-", env!("CARGO_PKG_VERSION"));

    /// Creates a new [`StringCowVisitor`].
    pub fn new() -> Self {
        Self(None)
    }

    /// Sets the value of the visitor.
    pub fn set(&mut self, value: StringCow<'de>) {
        self.0 = Some(value);
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::de::Visitor<'de> for StringCowVisitor<'de> {
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

    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if let Some(value) = self.0 {
            Ok(value)
        } else {
            deserializer.deserialize_str(self)
        }
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for StringCow<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_newtype_struct(StringCowVisitor::NEW_TYPE_NAME, StringCowVisitor::new())
    }
}
