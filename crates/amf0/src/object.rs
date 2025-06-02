//! This code is modified from <https://github.com/serde-rs/json/blob/v1.0.140/src/map.rs>
//!
//! A map of scuffle_bytes_util::StringCow to crate::Amf0Value.
//!
//! By default the map is backed by a [`BTreeMap`]. Enable the `preserve_order`
//! feature of scuffle_amf0 to use [`IndexMap`] instead.
//!
//! [`BTreeMap`]: std::collections::BTreeMap
//! [`IndexMap`]: indexmap::IndexMap

#[cfg(not(feature = "preserve_order"))]
use alloc::collections::{BTreeMap, btree_map};
use core::borrow::Borrow;
use core::fmt::{self, Debug};
use core::hash::Hash;
use core::iter::FusedIterator;
use core::ops;
use std::marker::PhantomData;

#[cfg(feature = "preserve_order")]
use indexmap::IndexMap;
use scuffle_bytes_util::StringCow;
use serde::de;

use crate::{Amf0Error, Amf0Value};

/// Represents a JSON key/value type.
#[derive(Clone, PartialEq)]
pub struct Amf0Object<'a> {
    map: MapImpl<StringCow<'a>, Amf0Value<'a>>,
}

#[cfg(not(feature = "preserve_order"))]
type MapImpl<K, V> = BTreeMap<K, V>;
#[cfg(feature = "preserve_order")]
type MapImpl<K, V> = IndexMap<K, V>;

impl<'a> Default for Amf0Object<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Amf0Object<'a> {
    /// Makes a new empty Map.
    #[inline]
    pub fn new() -> Self {
        Amf0Object { map: MapImpl::new() }
    }

    /// Makes a new empty Map with the given initial capacity.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Amf0Object {
            #[cfg(not(feature = "preserve_order"))]
            map: {
                // does not support with_capacity
                let _ = capacity;
                BTreeMap::new()
            },
            #[cfg(feature = "preserve_order")]
            map: IndexMap::with_capacity(capacity),
        }
    }

    /// Clears the map, removing all values.
    #[inline]
    pub fn clear(&mut self) {
        self.map.clear();
    }

    /// Returns a reference to the value corresponding to the key.
    ///
    /// The key may be any borrowed form of the map's key type, but the ordering
    /// on the borrowed form *must* match the ordering on the key type.
    #[inline]
    pub fn get<Q>(&self, key: &Q) -> Option<&Amf0Value<'a>>
    where
        StringCow<'a>: Borrow<Q>,
        Q: ?Sized + Ord + Eq + Hash,
    {
        self.map.get(key)
    }

    /// Returns true if the map contains a value for the specified key.
    ///
    /// The key may be any borrowed form of the map's key type, but the ordering
    /// on the borrowed form *must* match the ordering on the key type.
    #[inline]
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        StringCow<'a>: Borrow<Q>,
        Q: ?Sized + Ord + Eq + Hash,
    {
        self.map.contains_key(key)
    }

    /// Returns a mutable reference to the value corresponding to the key.
    ///
    /// The key may be any borrowed form of the map's key type, but the ordering
    /// on the borrowed form *must* match the ordering on the key type.
    #[inline]
    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut Amf0Value<'a>>
    where
        StringCow<'a>: Borrow<Q>,
        Q: ?Sized + Ord + Eq + Hash,
    {
        self.map.get_mut(key)
    }

    /// Returns the key-value pair matching the given key.
    ///
    /// The key may be any borrowed form of the map's key type, but the ordering
    /// on the borrowed form *must* match the ordering on the key type.
    #[inline]
    pub fn get_key_value<Q>(&self, key: &Q) -> Option<(&StringCow<'a>, &Amf0Value<'a>)>
    where
        StringCow<'a>: Borrow<Q>,
        Q: ?Sized + Ord + Eq + Hash,
    {
        self.map.get_key_value(key)
    }

    /// Inserts a key-value pair into the map.
    ///
    /// If the map did not have this key present, `None` is returned.
    ///
    /// If the map did have this key present, the value is updated, and the old
    /// value is returned.
    #[inline]
    pub fn insert(&mut self, k: StringCow<'a>, v: Amf0Value<'a>) -> Option<Amf0Value<'a>> {
        self.map.insert(k, v)
    }

    /// Insert a key-value pair in the map at the given index.
    ///
    /// If the map did not have this key present, `None` is returned.
    ///
    /// If the map did have this key present, the key is moved to the new
    /// position, the value is updated, and the old value is returned.
    #[cfg(feature = "preserve_order")]
    #[inline]
    pub fn shift_insert(&mut self, index: usize, k: StringCow<'a>, v: Amf0Value<'a>) -> Option<Amf0Value<'a>> {
        self.map.shift_insert(index, k, v)
    }

    /// Removes a key from the map, returning the value at the key if the key
    /// was previously in the map.
    ///
    /// The key may be any borrowed form of the map's key type, but the ordering
    /// on the borrowed form *must* match the ordering on the key type.
    ///
    /// If scuffle_amf0's "preserve_order" is enabled, `.remove(key)` is
    /// equivalent to [`.swap_remove(key)`][Self::swap_remove], replacing this
    /// entry's position with the last element. If you need to preserve the
    /// relative order of the keys in the map, use
    /// [`.shift_remove(key)`][Self::shift_remove] instead.
    #[inline]
    pub fn remove<Q>(&mut self, key: &Q) -> Option<Amf0Value<'a>>
    where
        StringCow<'a>: Borrow<Q>,
        Q: ?Sized + Ord + Eq + Hash,
    {
        #[cfg(feature = "preserve_order")]
        return self.swap_remove(key);
        #[cfg(not(feature = "preserve_order"))]
        return self.map.remove(key);
    }

    /// Removes a key from the map, returning the stored key and value if the
    /// key was previously in the map.
    ///
    /// The key may be any borrowed form of the map's key type, but the ordering
    /// on the borrowed form *must* match the ordering on the key type.
    ///
    /// If scuffle_amf0's "preserve_order" is enabled, `.remove_entry(key)` is
    /// equivalent to [`.swap_remove_entry(key)`][Self::swap_remove_entry],
    /// replacing this entry's position with the last element. If you need to
    /// preserve the relative order of the keys in the map, use
    /// [`.shift_remove_entry(key)`][Self::shift_remove_entry] instead.
    #[inline]
    pub fn remove_entry<Q>(&mut self, key: &Q) -> Option<(StringCow<'a>, Amf0Value<'a>)>
    where
        StringCow<'a>: Borrow<Q>,
        Q: ?Sized + Ord + Eq + Hash,
    {
        #[cfg(feature = "preserve_order")]
        return self.swap_remove_entry(key);
        #[cfg(not(feature = "preserve_order"))]
        return self.map.remove_entry(key);
    }

    /// Removes and returns the value corresponding to the key from the map.
    ///
    /// Like [`Vec::swap_remove`], the entry is removed by swapping it with the
    /// last element of the map and popping it off. This perturbs the position
    /// of what used to be the last element!
    ///
    /// [`Vec::swap_remove`]: std::vec::Vec::swap_remove
    #[cfg(feature = "preserve_order")]
    #[cfg_attr(docsrs, doc(cfg(feature = "preserve_order")))]
    #[inline]
    pub fn swap_remove<Q>(&mut self, key: &Q) -> Option<Amf0Value<'a>>
    where
        StringCow<'a>: Borrow<Q>,
        Q: ?Sized + Ord + Eq + Hash,
    {
        self.map.swap_remove(key)
    }

    /// Remove and return the key-value pair.
    ///
    /// Like [`Vec::swap_remove`], the entry is removed by swapping it with the
    /// last element of the map and popping it off. This perturbs the position
    /// of what used to be the last element!
    ///
    /// [`Vec::swap_remove`]: std::vec::Vec::swap_remove
    #[cfg(feature = "preserve_order")]
    #[cfg_attr(docsrs, doc(cfg(feature = "preserve_order")))]
    #[inline]
    pub fn swap_remove_entry<Q>(&mut self, key: &Q) -> Option<(StringCow<'a>, Amf0Value<'a>)>
    where
        StringCow<'a>: Borrow<Q>,
        Q: ?Sized + Ord + Eq + Hash,
    {
        self.map.swap_remove_entry(key)
    }

    /// Removes and returns the value corresponding to the key from the map.
    ///
    /// Like [`Vec::remove`], the entry is removed by shifting all of the
    /// elements that follow it, preserving their relative order. This perturbs
    /// the index of all of those elements!
    ///
    /// [`Vec::remove`]: std::vec::Vec::remove
    #[cfg(feature = "preserve_order")]
    #[cfg_attr(docsrs, doc(cfg(feature = "preserve_order")))]
    #[inline]
    pub fn shift_remove<Q>(&mut self, key: &Q) -> Option<Amf0Value<'a>>
    where
        StringCow<'a>: Borrow<Q>,
        Q: ?Sized + Ord + Eq + Hash,
    {
        self.map.shift_remove(key)
    }

    /// Remove and return the key-value pair.
    ///
    /// Like [`Vec::remove`], the entry is removed by shifting all of the
    /// elements that follow it, preserving their relative order. This perturbs
    /// the index of all of those elements!
    ///
    /// [`Vec::remove`]: std::vec::Vec::remove
    #[cfg(feature = "preserve_order")]
    #[cfg_attr(docsrs, doc(cfg(feature = "preserve_order")))]
    #[inline]
    pub fn shift_remove_entry<Q>(&mut self, key: &Q) -> Option<(StringCow<'a>, Amf0Value<'a>)>
    where
        StringCow<'a>: Borrow<Q>,
        Q: ?Sized + Ord + Eq + Hash,
    {
        self.map.shift_remove_entry(key)
    }

    /// Moves all elements from other into self, leaving other empty.
    #[inline]
    pub fn append(&mut self, other: &mut Self) {
        #[cfg(feature = "preserve_order")]
        self.map.extend(std::mem::take(&mut other.map));
        #[cfg(not(feature = "preserve_order"))]
        self.map.append(&mut other.map);
    }

    /// Gets the given key's corresponding entry in the map for in-place
    /// manipulation.
    pub fn entry(&mut self, key: impl Into<StringCow<'a>>) -> Entry<'_, 'a> {
        #[cfg(not(feature = "preserve_order"))]
        use alloc::collections::btree_map::Entry as EntryImpl;

        #[cfg(feature = "preserve_order")]
        use indexmap::map::Entry as EntryImpl;

        match self.map.entry(key.into()) {
            EntryImpl::Vacant(vacant) => Entry::Vacant(VacantEntry { vacant }),
            EntryImpl::Occupied(occupied) => Entry::Occupied(OccupiedEntry { occupied }),
        }
    }

    /// Returns the number of elements in the map.
    #[inline]
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Returns true if the map contains no elements.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Gets an iterator over the entries of the map.
    #[inline]
    pub fn iter(&self) -> Iter<'_, 'a> {
        Iter { iter: self.map.iter() }
    }

    /// Gets a mutable iterator over the entries of the map.
    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, 'a> {
        IterMut {
            iter: self.map.iter_mut(),
        }
    }

    /// Gets an iterator over the keys of the map.
    #[inline]
    pub fn keys(&self) -> Keys<'_, 'a> {
        Keys { iter: self.map.keys() }
    }

    /// Gets an iterator over the values of the map.
    #[inline]
    pub fn values(&self) -> Values<'_, 'a> {
        Values { iter: self.map.values() }
    }

    /// Gets an iterator over mutable values of the map.
    #[inline]
    pub fn values_mut(&mut self) -> ValuesMut<'_, 'a> {
        ValuesMut {
            iter: self.map.values_mut(),
        }
    }

    /// Gets an iterator over the values of the map.
    #[inline]
    pub fn into_values(self) -> IntoValues<'a> {
        IntoValues {
            iter: self.map.into_values(),
        }
    }

    /// Retains only the elements specified by the predicate.
    ///
    /// In other words, remove all pairs `(k, v)` such that `f(&k, &mut v)`
    /// returns `false`.
    #[inline]
    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&StringCow<'a>, &mut Amf0Value<'a>) -> bool,
    {
        self.map.retain(f);
    }

    /// Sorts this map's entries in-place using `str`'s usual ordering.
    ///
    /// If scuffle_amf0's "preserve_order" feature is not enabled, this method
    /// does no work because all JSON maps are always kept in a sorted state.
    ///
    /// If scuffle_amf0's "preserve_order" feature is enabled, this method
    /// destroys the original source order or insertion order of this map in
    /// favor of an alphanumerical order that matches how a BTreeMap with the
    /// same contents would be ordered. This takes **O(n log n + c)** time where
    /// _n_ is the length of the map and _c_ is the capacity.
    #[inline]
    pub fn sort_keys(&mut self) {
        #[cfg(feature = "preserve_order")]
        self.map.sort_unstable_keys();
    }
}

/// Access an element of this map. Panics if the given key is not present in the
/// map.
///
/// ```
/// use scuffle_amf0::{Amf0Object, Amf0Value};
/// use std::collections::BTreeMap;
/// use scuffle_bytes_util::StringCow;
///
/// let mut map = scuffle_amf0::Amf0Object::new();
/// map.insert(StringCow::from_ref("type"), Amf0Value::String("example".into()));
/// let obj = Amf0Object::from(map);
///
/// assert_eq!(obj["type"], Amf0Value::String("example".into()));
/// ```
impl<'a, Q> ops::Index<&Q> for Amf0Object<'a>
where
    StringCow<'a>: Borrow<Q>,
    Q: ?Sized + Ord + Eq + Hash,
{
    type Output = Amf0Value<'a>;

    fn index(&self, index: &Q) -> &Self::Output {
        self.map.index(index)
    }
}

/// Mutably access an element of this map. Panics if the given key is not
/// present in the map.
impl<'a, Q> ops::IndexMut<&Q> for Amf0Object<'a>
where
    StringCow<'a>: Borrow<Q>,
    Q: ?Sized + Ord + Eq + Hash,
{
    fn index_mut(&mut self, index: &Q) -> &mut Self::Output {
        self.map.get_mut(index).expect("no entry found for key")
    }
}

impl Debug for Amf0Object<'_> {
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.map.fmt(formatter)
    }
}

#[cfg(feature = "serde")]
impl serde::ser::Serialize for Amf0Object<'_> {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(self.len()))?;
        for (k, v) in self {
            map.serialize_entry(k, v)?;
        }
        map.end()
    }
}

#[cfg(feature = "serde")]
impl<'de> de::Deserialize<'de> for Amf0Object<'de> {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> de::Visitor<'de> for Visitor {
            type Value = Amf0Object<'de>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a map")
            }

            #[inline]
            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Amf0Object::new())
            }

            #[inline]
            fn visit_map<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
            where
                V: de::MapAccess<'de>,
            {
                let mut values = Amf0Object::new();

                while let Some((key, value)) = visitor.next_entry()? {
                    values.insert(key, value);
                }

                Ok(values)
            }
        }

        deserializer.deserialize_map(Visitor)
    }
}

impl<'a> FromIterator<(StringCow<'a>, Amf0Value<'a>)> for Amf0Object<'a> {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (StringCow<'a>, Amf0Value<'a>)>,
    {
        Amf0Object {
            map: FromIterator::from_iter(iter),
        }
    }
}

impl<'a> Extend<(StringCow<'a>, Amf0Value<'a>)> for Amf0Object<'a> {
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = (StringCow<'a>, Amf0Value<'a>)>,
    {
        self.map.extend(iter);
    }
}

macro_rules! delegate_iterator {
    (($name:ident $($generics:tt)*) => $item:ty) => {
        impl $($generics)* Iterator for $name $($generics)* {
            type Item = $item;
            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                self.iter.next()
            }
            #[inline]
            fn size_hint(&self) -> (usize, Option<usize>) {
                self.iter.size_hint()
            }
        }

        impl $($generics)* DoubleEndedIterator for $name $($generics)* {
            #[inline]
            fn next_back(&mut self) -> Option<Self::Item> {
                self.iter.next_back()
            }
        }

        impl $($generics)* ExactSizeIterator for $name $($generics)* {
            #[inline]
            fn len(&self) -> usize {
                self.iter.len()
            }
        }

        impl $($generics)* FusedIterator for $name $($generics)* {}
    }
}

#[cfg(feature = "serde")]
impl<'de> de::IntoDeserializer<'de, Amf0Error> for Amf0Object<'de> {
    type Deserializer = Self;

    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

#[cfg(feature = "serde")]
impl<'de> de::IntoDeserializer<'de, Amf0Error> for &'de Amf0Object<'de> {
    type Deserializer = Self;

    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::de::Deserializer<'de> for Amf0Object<'de> {
    type Error = Amf0Error;

    serde::forward_to_deserialize_any! {
        bool f64 f32 char str string unit
        i8 i16 i32 i64 u8 u16 u32 u64
        seq map newtype_struct tuple
        struct enum ignored_any identifier
        bytes byte_buf option unit_struct
        tuple_struct
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_map(Amf0MapAccess {
            iter: self.into_iter(),
            _phantom: PhantomData,
            value: None,
        })
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::de::Deserializer<'de> for &'de Amf0Object<'de> {
    type Error = Amf0Error;

    serde::forward_to_deserialize_any! {
        bool f64 f32 char str string unit
        i8 i16 i32 i64 u8 u16 u32 u64
        seq map newtype_struct tuple
        struct enum ignored_any identifier
        bytes byte_buf option unit_struct
        tuple_struct
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_map(Amf0MapAccess {
            iter: self.into_iter(),
            _phantom: PhantomData,
            value: None,
        })
    }
}

#[cfg(feature = "serde")]
struct Amf0MapAccess<I, K, V> {
    iter: I,
    value: Option<V>,
    _phantom: PhantomData<K>,
}

#[cfg(feature = "serde")]
impl<'de, I, K, V> serde::de::MapAccess<'de> for Amf0MapAccess<I, K, V>
where
    I: Iterator<Item = (K, V)>,
    K: serde::de::IntoDeserializer<'de, Amf0Error>,
    V: serde::de::IntoDeserializer<'de, Amf0Error>,
{
    type Error = Amf0Error;

    fn next_key_seed<S>(&mut self, seed: S) -> Result<Option<S::Value>, Self::Error>
    where
        S: serde::de::DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some((key, value)) => {
                self.value = Some(value);
                seed.deserialize(key.into_deserializer()).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<S>(&mut self, seed: S) -> Result<S::Value, Self::Error>
    where
        S: serde::de::DeserializeSeed<'de>,
    {
        seed.deserialize(self.value.take().unwrap().into_deserializer())
    }
}

//////////////////////////////////////////////////////////////////////////////

/// A view into a single entry in a map, which may either be vacant or occupied.
/// This enum is constructed from the [`entry`] method on [`Amf0Object`].
///
/// [`entry`]: Amf0Object::entry
pub enum Entry<'a, 'b> {
    /// A vacant Entry.
    Vacant(VacantEntry<'a, 'b>),
    /// An occupied Entry.
    Occupied(OccupiedEntry<'a, 'b>),
}

/// A vacant Entry. It is part of the [`Entry`] enum.
pub struct VacantEntry<'a, 'b> {
    vacant: VacantEntryImpl<'a, 'b>,
}

/// An occupied Entry. It is part of the [`Entry`] enum.
pub struct OccupiedEntry<'a, 'b> {
    occupied: OccupiedEntryImpl<'a, 'b>,
}

#[cfg(not(feature = "preserve_order"))]
type VacantEntryImpl<'a, 'b> = btree_map::VacantEntry<'a, StringCow<'b>, Amf0Value<'b>>;
#[cfg(feature = "preserve_order")]
type VacantEntryImpl<'a, 'b> = indexmap::map::VacantEntry<'a, StringCow<'b>, Amf0Value<'b>>;

#[cfg(not(feature = "preserve_order"))]
type OccupiedEntryImpl<'a, 'b> = btree_map::OccupiedEntry<'a, StringCow<'b>, Amf0Value<'b>>;
#[cfg(feature = "preserve_order")]
type OccupiedEntryImpl<'a, 'b> = indexmap::map::OccupiedEntry<'a, StringCow<'b>, Amf0Value<'b>>;

impl<'a, 'b> Entry<'a, 'b> {
    /// Returns a reference to this entry's key.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut map = scuffle_amf0::Amf0Object::new();
    /// assert_eq!(map.entry("serde").key(), &"serde");
    /// ```
    pub fn key(&self) -> &StringCow<'b> {
        match self {
            Entry::Vacant(e) => e.key(),
            Entry::Occupied(e) => e.key(),
        }
    }

    /// Ensures a value is in the entry by inserting the default if empty, and
    /// returns a mutable reference to the value in the entry.
    pub fn or_insert(self, default: Amf0Value<'b>) -> &'a mut Amf0Value<'b> {
        match self {
            Entry::Vacant(entry) => entry.insert(default),
            Entry::Occupied(entry) => entry.into_mut(),
        }
    }

    /// Ensures a value is in the entry by inserting the result of the default
    /// function if empty, and returns a mutable reference to the value in the
    /// entry.
    pub fn or_insert_with<F>(self, default: F) -> &'a mut Amf0Value<'b>
    where
        F: FnOnce() -> Amf0Value<'b>,
    {
        match self {
            Entry::Vacant(entry) => entry.insert(default()),
            Entry::Occupied(entry) => entry.into_mut(),
        }
    }

    /// Provides in-place mutable access to an occupied entry before any
    /// potential inserts into the map.
    pub fn and_modify<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut Amf0Value<'b>),
    {
        match self {
            Entry::Occupied(mut entry) => {
                f(entry.get_mut());
                Entry::Occupied(entry)
            }
            Entry::Vacant(entry) => Entry::Vacant(entry),
        }
    }
}

impl<'a, 'b> VacantEntry<'a, 'b> {
    /// Gets a reference to the key that would be used when inserting a value
    /// through the VacantEntry.
    ///
    /// # Examples
    ///
    /// ```
    /// use scuffle_amf0::object::Entry;
    ///
    /// let mut map = scuffle_amf0::Amf0Object::new();
    ///
    /// match map.entry("serde") {
    ///     Entry::Vacant(vacant) => {
    ///         assert_eq!(vacant.key(), &"serde");
    ///     }
    ///     Entry::Occupied(_) => unimplemented!(),
    /// }
    /// ```
    #[inline]
    pub fn key(&self) -> &StringCow<'b> {
        self.vacant.key()
    }

    /// Sets the value of the entry with the VacantEntry's key, and returns a
    /// mutable reference to it.
    #[inline]
    pub fn insert(self, value: Amf0Value<'b>) -> &'a mut Amf0Value<'b> {
        self.vacant.insert(value)
    }
}

impl<'a, 'b> OccupiedEntry<'a, 'b> {
    /// Gets a reference to the key in the entry.
    #[inline]
    pub fn key(&self) -> &StringCow<'b> {
        self.occupied.key()
    }

    /// Gets a reference to the value in the entry.
    #[inline]
    pub fn get(&self) -> &Amf0Value<'b> {
        self.occupied.get()
    }

    /// Gets a mutable reference to the value in the entry.
    #[inline]
    pub fn get_mut(&mut self) -> &mut Amf0Value<'b> {
        self.occupied.get_mut()
    }

    /// Converts the entry into a mutable reference to its value.
    #[inline]
    pub fn into_mut(self) -> &'a mut Amf0Value<'b> {
        self.occupied.into_mut()
    }

    /// Sets the value of the entry with the `OccupiedEntry`'s key, and returns
    /// the entry's old value.
    #[inline]
    pub fn insert(&mut self, value: Amf0Value<'b>) -> Amf0Value<'b> {
        self.occupied.insert(value)
    }

    /// Takes the value of the entry out of the map, and returns it.
    ///
    /// If scuffle_amf0's "preserve_order" is enabled, `.remove()` is
    /// equivalent to [`.swap_remove()`][Self::swap_remove], replacing this
    /// entry's position with the last element. If you need to preserve the
    /// relative order of the keys in the map, use
    /// [`.shift_remove()`][Self::shift_remove] instead.
    #[inline]
    pub fn remove(self) -> Amf0Value<'b> {
        #[cfg(feature = "preserve_order")]
        return self.swap_remove();
        #[cfg(not(feature = "preserve_order"))]
        return self.occupied.remove();
    }

    /// Takes the value of the entry out of the map, and returns it.
    ///
    /// Like [`Vec::swap_remove`], the entry is removed by swapping it with the
    /// last element of the map and popping it off. This perturbs the position
    /// of what used to be the last element!
    ///
    /// [`Vec::swap_remove`]: std::vec::Vec::swap_remove
    #[cfg(feature = "preserve_order")]
    #[cfg_attr(docsrs, doc(cfg(feature = "preserve_order")))]
    #[inline]
    pub fn swap_remove(self) -> Amf0Value<'b> {
        self.occupied.swap_remove()
    }

    /// Takes the value of the entry out of the map, and returns it.
    ///
    /// Like [`Vec::remove`], the entry is removed by shifting all of the
    /// elements that follow it, preserving their relative order. This perturbs
    /// the index of all of those elements!
    ///
    /// [`Vec::remove`]: std::vec::Vec::remove
    #[cfg(feature = "preserve_order")]
    #[cfg_attr(docsrs, doc(cfg(feature = "preserve_order")))]
    #[inline]
    pub fn shift_remove(self) -> Amf0Value<'b> {
        self.occupied.shift_remove()
    }

    /// Removes the entry from the map, returning the stored key and value.
    ///
    /// If scuffle_amf0's "preserve_order" is enabled, `.remove_entry()` is
    /// equivalent to [`.swap_remove_entry()`][Self::swap_remove_entry],
    /// replacing this entry's position with the last element. If you need to
    /// preserve the relative order of the keys in the map, use
    /// [`.shift_remove_entry()`][Self::shift_remove_entry] instead.
    #[inline]
    pub fn remove_entry(self) -> (StringCow<'b>, Amf0Value<'b>) {
        #[cfg(feature = "preserve_order")]
        return self.swap_remove_entry();
        #[cfg(not(feature = "preserve_order"))]
        return self.occupied.remove_entry();
    }

    /// Removes the entry from the map, returning the stored key and value.
    ///
    /// Like [`Vec::swap_remove`], the entry is removed by swapping it with the
    /// last element of the map and popping it off. This perturbs the position
    /// of what used to be the last element!
    ///
    /// [`Vec::swap_remove`]: std::vec::Vec::swap_remove
    #[cfg(feature = "preserve_order")]
    #[cfg_attr(docsrs, doc(cfg(feature = "preserve_order")))]
    #[inline]
    pub fn swap_remove_entry(self) -> (StringCow<'b>, Amf0Value<'b>) {
        self.occupied.swap_remove_entry()
    }

    /// Removes the entry from the map, returning the stored key and value.
    ///
    /// Like [`Vec::remove`], the entry is removed by shifting all of the
    /// elements that follow it, preserving their relative order. This perturbs
    /// the index of all of those elements!
    ///
    /// [`Vec::remove`]: std::vec::Vec::remove
    #[cfg(feature = "preserve_order")]
    #[cfg_attr(docsrs, doc(cfg(feature = "preserve_order")))]
    #[inline]
    pub fn shift_remove_entry(self) -> (StringCow<'b>, Amf0Value<'b>) {
        self.occupied.shift_remove_entry()
    }
}

//////////////////////////////////////////////////////////////////////////////

impl<'a, 'b> IntoIterator for &'a Amf0Object<'b> {
    type IntoIter = Iter<'a, 'b>;
    type Item = (&'a StringCow<'b>, &'a Amf0Value<'b>);

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        Iter { iter: self.map.iter() }
    }
}

/// An iterator over a scuffle_amf0::Amf0Object's entries.
pub struct Iter<'a, 'b> {
    iter: IterImpl<'a, 'b>,
}

#[cfg(not(feature = "preserve_order"))]
type IterImpl<'a, 'b> = btree_map::Iter<'a, StringCow<'b>, Amf0Value<'b>>;
#[cfg(feature = "preserve_order")]
type IterImpl<'a, 'b> = indexmap::map::Iter<'a, StringCow<'b>, Amf0Value<'b>>;

delegate_iterator!((Iter<'a, 'b>) => (&'a StringCow<'b>, &'a Amf0Value<'b>));

//////////////////////////////////////////////////////////////////////////////

impl<'a, 'b> IntoIterator for &'a mut Amf0Object<'b> {
    type IntoIter = IterMut<'a, 'b>;
    type Item = (&'a StringCow<'b>, &'a mut Amf0Value<'b>);

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IterMut {
            iter: self.map.iter_mut(),
        }
    }
}

/// A mutable iterator over a scuffle_amf0::Amf0Object's entries.
pub struct IterMut<'a, 'b> {
    iter: IterMutImpl<'a, 'b>,
}

#[cfg(not(feature = "preserve_order"))]
type IterMutImpl<'a, 'b> = btree_map::IterMut<'a, StringCow<'b>, Amf0Value<'b>>;
#[cfg(feature = "preserve_order")]
type IterMutImpl<'a, 'b> = indexmap::map::IterMut<'a, StringCow<'b>, Amf0Value<'b>>;

delegate_iterator!((IterMut<'a, 'b>) => (&'a StringCow<'b>, &'a mut Amf0Value<'b>));

//////////////////////////////////////////////////////////////////////////////

impl<'a> IntoIterator for Amf0Object<'a> {
    type IntoIter = IntoIter<'a>;
    type Item = (StringCow<'a>, Amf0Value<'a>);

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            iter: self.map.into_iter(),
        }
    }
}

/// An owning iterator over a scuffle_amf0::Amf0Object's entries.
pub struct IntoIter<'a> {
    iter: IntoIterImpl<'a>,
}

#[cfg(not(feature = "preserve_order"))]
type IntoIterImpl<'a> = btree_map::IntoIter<StringCow<'a>, Amf0Value<'a>>;
#[cfg(feature = "preserve_order")]
type IntoIterImpl<'a> = indexmap::map::IntoIter<StringCow<'a>, Amf0Value<'a>>;

delegate_iterator!((IntoIter<'a>) => (StringCow<'a>, Amf0Value<'a>));

//////////////////////////////////////////////////////////////////////////////

/// An iterator over a scuffle_amf0::Amf0Object's keys.
pub struct Keys<'a, 'b> {
    iter: KeysImpl<'a, 'b>,
}

#[cfg(not(feature = "preserve_order"))]
type KeysImpl<'a, 'b> = btree_map::Keys<'a, StringCow<'b>, Amf0Value<'b>>;
#[cfg(feature = "preserve_order")]
type KeysImpl<'a, 'b> = indexmap::map::Keys<'a, StringCow<'b>, Amf0Value<'b>>;

delegate_iterator!((Keys<'a, 'b>) => &'a StringCow<'b>);

//////////////////////////////////////////////////////////////////////////////

/// An iterator over a scuffle_amf0::Amf0Object's values.
pub struct Values<'a, 'b> {
    iter: ValuesImpl<'a, 'b>,
}

#[cfg(not(feature = "preserve_order"))]
type ValuesImpl<'a, 'b> = btree_map::Values<'a, StringCow<'b>, Amf0Value<'b>>;
#[cfg(feature = "preserve_order")]
type ValuesImpl<'a, 'b> = indexmap::map::Values<'a, StringCow<'b>, Amf0Value<'b>>;

delegate_iterator!((Values<'a, 'b>) => &'a Amf0Value<'b>);

//////////////////////////////////////////////////////////////////////////////

/// A mutable iterator over a scuffle_amf0::Amf0Object's values.
pub struct ValuesMut<'a, 'b> {
    iter: ValuesMutImpl<'a, 'b>,
}

#[cfg(not(feature = "preserve_order"))]
type ValuesMutImpl<'a, 'b> = btree_map::ValuesMut<'a, StringCow<'b>, Amf0Value<'b>>;
#[cfg(feature = "preserve_order")]
type ValuesMutImpl<'a, 'b> = indexmap::map::ValuesMut<'a, StringCow<'b>, Amf0Value<'b>>;

delegate_iterator!((ValuesMut<'a, 'b>) => &'a mut Amf0Value<'b>);

//////////////////////////////////////////////////////////////////////////////

/// An owning iterator over a scuffle_amf0::Amf0Object's values.
pub struct IntoValues<'a> {
    iter: IntoValuesImpl<'a>,
}

#[cfg(not(feature = "preserve_order"))]
type IntoValuesImpl<'a> = btree_map::IntoValues<StringCow<'a>, Amf0Value<'a>>;
#[cfg(feature = "preserve_order")]
type IntoValuesImpl<'a> = indexmap::map::IntoValues<StringCow<'a>, Amf0Value<'a>>;

delegate_iterator!((IntoValues<'a>) => Amf0Value<'a>);
