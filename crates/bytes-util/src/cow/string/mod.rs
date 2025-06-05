use std::borrow::Cow;
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::str::Utf8Error;

use bytestring::ByteString;

use crate::BytesCow;

#[cfg(feature = "serde")]
pub(crate) mod serde;

/// A [`Cow`] type for strings.
#[derive(Clone, Eq)]
pub enum StringCow<'a> {
    /// A borrowed [`ByteString`] object.
    Ref(&'a str),
    /// A staticly borrowed [`ByteString`] object.
    StaticRef(&'static str),
    /// An owned [`String`] object.
    String(String),
    /// An owned [`ByteString`] object.
    Bytes(ByteString),
}

impl Debug for StringCow<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self.as_str(), f)
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

    /// Converts this [`StringCow`] into an owned version with a static lifetime.
    pub fn into_owned(self) -> StringCow<'static> {
        match self {
            StringCow::Ref(slice) => StringCow::from(slice.to_owned()),
            StringCow::StaticRef(slice) => StringCow::StaticRef(slice),
            StringCow::String(string) => StringCow::String(string),
            StringCow::Bytes(bytes) => StringCow::Bytes(bytes),
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

    /// Returns the length of this [`StringCow`].
    pub fn len(&self) -> usize {
        match self {
            StringCow::Ref(slice) => slice.len(),
            StringCow::StaticRef(slice) => slice.len(),
            StringCow::String(string) => string.len(),
            StringCow::Bytes(bytes) => bytes.len(),
        }
    }

    /// Returns `true` if this [`StringCow`] is empty.
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Ref(slice) => slice.is_empty(),
            Self::StaticRef(slice) => slice.is_empty(),
            Self::String(string) => string.is_empty(),
            Self::Bytes(bytes) => bytes.is_empty(),
        }
    }
}

impl PartialEq<str> for StringCow<'_> {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
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

impl Ord for StringCow<'_> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_str().cmp(other.as_str())
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

impl Display for StringCow<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self.as_str(), f)
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

impl<'a> TryFrom<BytesCow<'a>> for StringCow<'a> {
    type Error = Utf8Error;

    fn try_from(value: BytesCow<'a>) -> Result<Self, Self::Error> {
        let bytes = ByteString::try_from(value.into_bytes())?;
        Ok(Self::from_bytes(bytes))
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use bytestring::ByteString;

    use super::StringCow;

    #[test]
    fn constructors() {
        let cow = StringCow::default();
        assert_eq!(cow.as_str(), "");

        let cow = StringCow::from_static("hello");
        assert_eq!(cow.as_str(), "hello");

        let cow = StringCow::from_ref("world");
        assert_eq!(cow.as_str(), "world");

        let cow = StringCow::from_string(String::from("foo"));
        assert_eq!(cow.as_str(), "foo");
        let cow = StringCow::from(String::from("bar"));
        assert_eq!(cow.as_str(), "bar");

        let cow = StringCow::from_bytes(ByteString::from_static("foo"));
        assert_eq!(cow.as_str(), "foo");
        let cow = StringCow::from(ByteString::from_static("foo"));
        assert_eq!(cow.as_str(), "foo");

        let cow = StringCow::from_cow(std::borrow::Cow::Borrowed("bar"));
        assert_eq!(cow.as_str(), "bar");
        let cow = StringCow::from_cow(std::borrow::Cow::Owned(String::from("baz")));
        assert_eq!(cow.as_str(), "baz");
        let cow = StringCow::from(std::borrow::Cow::Owned(String::from("qux")));
        assert_eq!(cow.as_str(), "qux");
    }

    #[test]
    fn into_bytes() {
        let cow = StringCow::from_static("hello");
        assert_eq!(cow.into_bytes(), ByteString::from_static("hello"));

        let cow = StringCow::from_ref("world");
        assert_eq!(cow.into_bytes(), ByteString::from_static("world"));

        let cow = StringCow::from_string(String::from("foo"));
        assert_eq!(cow.into_bytes(), ByteString::from_static("foo"));

        let cow = StringCow::from_bytes(ByteString::from_static("foo"));
        assert_eq!(cow.into_bytes(), ByteString::from_static("foo"));

        let cow = StringCow::from_cow(std::borrow::Cow::Borrowed("bar"));
        assert_eq!(cow.into_bytes(), ByteString::from_static("bar"));

        let cow = StringCow::from_cow(std::borrow::Cow::Owned(String::from("baz")));
        assert_eq!(cow.into_bytes(), ByteString::from_static("baz"));
    }

    #[test]
    fn as_ref() {
        let cow = StringCow::from_static("hello");
        assert_eq!(cow.as_ref(), "hello");

        let cow = StringCow::from_ref("world");
        assert_eq!(cow.as_ref(), "world");

        let cow = StringCow::from_string(String::from("foo"));
        assert_eq!(cow.as_ref(), "foo");

        let cow = StringCow::from_bytes(ByteString::from_static("foo"));
        assert_eq!(cow.as_ref(), "foo");
    }

    #[test]
    fn into_owned() {
        let cow = StringCow::from_static("hello");
        assert_eq!(cow.into_owned().as_str(), "hello");

        let cow = StringCow::from_ref("world");
        assert_eq!(cow.into_owned().as_str(), "world");

        let cow = StringCow::from_string(String::from("foo"));
        assert_eq!(cow.into_owned().as_str(), "foo");

        let cow = StringCow::from_bytes(ByteString::from_static("foo"));
        assert_eq!(cow.into_owned().as_str(), "foo");
    }

    #[test]
    fn partial_eq() {
        let cow = StringCow::from_static("hello");
        assert!(cow == "hello");
        assert!(cow != "world");

        let cow = StringCow::from_ref("world");
        assert!(cow == "world");
        assert!(cow != "hello");

        let cow = StringCow::from_string(String::from("foo"));
        assert!(cow == "foo");
        assert!(cow != "bar");
    }

    #[test]
    fn hash() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        "hello".hash(&mut hasher);
        let expected_hash = hasher.finish();

        let cow = StringCow::from_static("hello");
        let mut hasher = DefaultHasher::new();
        cow.hash(&mut hasher);
        assert_eq!(hasher.finish(), expected_hash);
    }

    #[test]
    fn partial_ord() {
        let cow1 = StringCow::from_static("hello");
        let cow2 = StringCow::from_static("world");
        assert!(cow1 < cow2);

        let cow3 = StringCow::from_ref("foo");
        let cow4 = StringCow::from_string(String::from("bar"));
        assert!(cow3 > cow4);
    }

    #[test]
    fn display() {
        let cow = StringCow::from_ref("hello");
        let fmt = format!("{cow}");
        assert_eq!(fmt, "hello");

        let cow = StringCow::from_static("hello");
        let fmt = format!("{cow}");
        assert_eq!(fmt, "hello");

        let cow = StringCow::from_string(String::from("world"));
        let fmt = format!("{cow}");
        assert_eq!(fmt, "world");

        let cow = StringCow::from_bytes(ByteString::from_static("foo"));
        let fmt = format!("{cow}");
        assert_eq!(fmt, "foo");
    }
}
