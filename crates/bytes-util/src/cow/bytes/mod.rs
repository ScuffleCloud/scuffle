use std::borrow::Cow;
use std::fmt::Debug;
use std::hash::Hash;

use bytes::Bytes;

#[cfg(feature = "serde")]
pub(crate) mod serde;

/// A [`Cow`] type for bytes.
#[derive(Clone, Eq)]
pub enum BytesCow<'a> {
    /// A borrowed [`Bytes`] object.
    Slice(&'a [u8]),
    /// A staticly borrowed [`Bytes`] object.
    StaticSlice(&'static [u8]),
    /// An owned [`Vec`] of bytes.
    Vec(Vec<u8>),
    /// An owned [`Bytes`] object.
    Bytes(Bytes),
}

impl Debug for BytesCow<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Slice(slice) => Debug::fmt(slice, f),
            Self::StaticSlice(slice) => Debug::fmt(slice, f),
            Self::Vec(bytes) => Debug::fmt(bytes, f),
            Self::Bytes(bytes) => Debug::fmt(bytes, f),
        }
    }
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

    /// Returns the length of this [`BytesCow`].
    pub fn len(&self) -> usize {
        match self {
            Self::Slice(slice) => slice.len(),
            Self::StaticSlice(slice) => slice.len(),
            Self::Vec(bytes) => bytes.len(),
            Self::Bytes(bytes) => bytes.len(),
        }
    }

    /// Returns `true` if this [`BytesCow`] is empty.
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Slice(slice) => slice.is_empty(),
            Self::StaticSlice(slice) => slice.is_empty(),
            Self::Vec(bytes) => bytes.is_empty(),
            Self::Bytes(bytes) => bytes.is_empty(),
        }
    }

    /// Pads the bytes to a [`u64`].
    ///
    /// The bytes are expected to be in big-endian order.
    ///
    /// # Panics
    ///
    /// The caller must ensure that the length of the bytes is less than or equal to 8,
    /// otherwise this function will panic.
    pub fn pad_to_u64_be(&self) -> u64 {
        assert!(self.len() <= 8);

        // We copy the bytes into an 8 byte array and convert it to a u64
        let mut buf = [0u8; 8];
        buf[8 - self.len()..].copy_from_slice(self.as_bytes());
        u64::from_be_bytes(buf)
    }

    /// Pads the bytes to a [`u32`].
    ///
    /// The bytes are expected to be in big-endian order.
    ///
    /// # Panics
    ///
    /// The caller must ensure that the length of the bytes is less than or equal to 4,
    /// otherwise this function will panic.
    pub fn pad_to_u32_be(&self) -> u32 {
        assert!(self.len() <= 4);

        // We copy the bytes into a 4 byte array and convert it to a u32
        let mut buf = [0u8; 4];
        buf[4 - self.len()..].copy_from_slice(self.as_bytes());
        u32::from_be_bytes(buf)
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

impl Hash for BytesCow<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_bytes().hash(state);
    }
}

impl AsRef<[u8]> for BytesCow<'_> {
    fn as_ref(&self) -> &[u8] {
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

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use super::BytesCow;

    #[test]
    fn constructors() {
        let cow = BytesCow::default();
        assert_eq!(cow.as_bytes(), b"");

        let cow = BytesCow::from_static(b"hello");
        assert_eq!(cow.as_bytes(), b"hello");

        let cow = BytesCow::from_slice(b"world");
        assert_eq!(cow.as_bytes(), b"world");

        let cow = BytesCow::from_vec(vec![1, 2, 3]);
        assert_eq!(cow.as_bytes(), &[1, 2, 3]);
        let cow = BytesCow::from(vec![1, 2, 3]);
        assert_eq!(cow.as_bytes(), &[1, 2, 3]);

        let cow = BytesCow::from_bytes(bytes::Bytes::from_static(b"foo"));
        assert_eq!(cow.as_bytes(), b"foo");
        let cow = BytesCow::from(bytes::Bytes::from(vec![7, 8, 9]));
        assert_eq!(cow.as_bytes(), &[7, 8, 9]);

        let cow = BytesCow::from_cow(std::borrow::Cow::Borrowed(b"bar"));
        assert_eq!(cow.as_bytes(), b"bar");
        let cow = BytesCow::from_cow(std::borrow::Cow::Owned(vec![10, 11, 12]));
        assert_eq!(cow.as_bytes(), &[10, 11, 12]);
        let cow = BytesCow::from(std::borrow::Cow::Owned(vec![4, 5, 6]));
        assert_eq!(cow.as_bytes(), &[4, 5, 6]);

        let cow = BytesCow::from(&b"hello world"[..]);
        assert_eq!(cow.as_bytes(), b"hello world");
    }

    #[test]
    fn into_bytes() {
        let cow = BytesCow::from_static(b"hello");
        assert_eq!(cow.into_bytes(), bytes::Bytes::from_static(b"hello"));

        let cow = BytesCow::from_slice(b"world");
        assert_eq!(cow.into_bytes(), bytes::Bytes::from_static(b"world"));

        let cow = BytesCow::from_vec(vec![1, 2, 3]);
        assert_eq!(cow.into_bytes(), bytes::Bytes::from(vec![1, 2, 3]));

        let cow = BytesCow::from_bytes(bytes::Bytes::from_static(b"foo"));
        assert_eq!(cow.into_bytes(), bytes::Bytes::from_static(b"foo"));

        let cow = BytesCow::from_cow(std::borrow::Cow::Borrowed(b"bar"));
        assert_eq!(cow.into_bytes(), bytes::Bytes::from_static(b"bar"));

        let cow = BytesCow::from_cow(std::borrow::Cow::Owned(vec![10, 11, 12]));
        assert_eq!(cow.into_bytes(), bytes::Bytes::from(vec![10, 11, 12]));
    }

    #[test]
    fn as_ref() {
        let cow = BytesCow::from_static(b"hello");
        assert_eq!(cow.as_ref(), b"hello");

        let cow = BytesCow::from_slice(b"world");
        assert_eq!(cow.as_ref(), b"world");

        let cow = BytesCow::from_vec(vec![1, 2, 3]);
        assert_eq!(cow.as_ref(), &[1, 2, 3]);

        let cow = BytesCow::from_bytes(bytes::Bytes::from_static(b"foo"));
        assert_eq!(cow.as_ref(), b"foo");
    }

    #[test]
    fn partial_eq() {
        let cow = BytesCow::from_static(b"hello");
        assert!(cow == b"hello");
        assert!(cow != b"world");

        let cow = BytesCow::from_slice(b"world");
        assert!(cow == b"world");
        assert!(cow != b"hello");

        let cow = BytesCow::from_vec(vec![1, 2, 3]);
        assert!(cow == [1, 2, 3]);
        assert!(cow != [4, 5, 6]);
    }

    #[test]
    fn hash() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        b"hello".hash(&mut hasher);
        let expected_hash = hasher.finish();

        let cow = BytesCow::from_static(b"hello");
        let mut hasher = DefaultHasher::new();
        cow.hash(&mut hasher);
        assert_eq!(hasher.finish(), expected_hash);
    }
}
