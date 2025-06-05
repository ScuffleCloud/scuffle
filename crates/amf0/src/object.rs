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

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use scuffle_bytes_util::StringCow;
    use serde::Deserialize;

    use super::Entry;
    use crate::{Amf0Object, Amf0Value};

    #[test]
    fn test_default_map_is_empty() {
        let obj: Amf0Object = Amf0Object::default();
        assert!(obj.map.is_empty(), "Default Amf0Object should be empty");
    }

    #[cfg(feature = "preserve_order")]
    #[test]
    fn test_capacity_reserves_space_in_preserve_order() {
        let capacity = 16;
        let obj: Amf0Object = Amf0Object::with_capacity(capacity);

        use indexmap::map::IndexMap;
        let map: &IndexMap<_, _> = &obj.map;
        assert!(
            map.capacity() >= capacity,
            "IndexMap should reserve at least the requested capacity"
        );
    }

    #[cfg(not(feature = "preserve_order"))]
    #[test]
    fn test_capacity_ignored_without_preserve_order() {
        let capacity = 16;
        let obj: Amf0Object = Amf0Object::with_capacity(capacity);

        let key = StringCow::from("foo");
        let value = Amf0Value::String(StringCow::from("bar"));
        assert!(obj.map.insert(key.clone(), value.clone()).is_none());
        assert_eq!(obj.map.get(&key), Some(&value));
    }

    #[test]
    fn test_clear() {
        let mut obj = Amf0Object::new();

        obj.map.insert(StringCow::from("key1"), Amf0Value::Number(1.0));
        obj.map
            .insert(StringCow::from("key2"), Amf0Value::String(StringCow::from("value")));
        obj.map.insert(StringCow::from("key3"), Amf0Value::Boolean(true));

        assert!(!obj.map.is_empty(), "Map should not be empty before clear");
        obj.clear();
        assert!(obj.map.is_empty(), "Map should be empty after clear");
    }

    #[test]
    fn test_contains_key() {
        let mut obj = Amf0Object::new();

        let existing_key = StringCow::from("present");
        let missing_key = "missing";

        obj.map.insert(existing_key.clone(), Amf0Value::Boolean(true));

        assert!(obj.contains_key(&existing_key), "Expected key to be found using StringCow");
        assert!(obj.contains_key("present"), "Expected key to be found using &str");
        assert!(!obj.contains_key(missing_key), "Expected missing key to not be found");
    }

    #[test]
    fn test_get_mut_valid() {
        let mut obj = Amf0Object::new();
        obj.map
            .insert(StringCow::from("username"), Amf0Value::String(StringCow::from("old_name")));

        if let Some(value) = obj.get_mut("username") {
            *value = Amf0Value::String(StringCow::from("new_name"));
        } else {
            panic!("Expected Some(&mut Amf0Value) for existing key");
        }

        assert_eq!(obj.map.get("username"), Some(&Amf0Value::String(StringCow::from("new_name"))));
    }

    #[test]
    fn test_get_mut_invalid() {
        let mut obj = Amf0Object::new();
        assert!(obj.get_mut("nonexistent_key").is_none());
    }

    #[test]
    fn test_get_key_value_valid() {
        let mut obj = Amf0Object::new();
        let key = StringCow::from("foo");
        let value = Amf0Value::Number(3.21);

        obj.map.insert(key.clone(), value.clone());

        if let Some((found_key, found_value)) = obj.get_key_value("foo") {
            assert_eq!(found_key, &key);
            assert_eq!(found_value, &value);
        } else {
            panic!("Expected Some((key, value)) for existing key");
        }
    }

    #[test]
    fn test_get_key_value_invalid() {
        let obj = Amf0Object::new();
        assert!(obj.get_key_value("missing").is_none());
    }

    #[cfg(feature = "preserve_order")]
    #[test]
    fn test_shift_insert_inserts_and_shifts_correctly() {
        let mut obj = Amf0Object::new();

        obj.map.insert(StringCow::from("a"), Amf0Value::Number(1.0));
        obj.map.insert(StringCow::from("b"), Amf0Value::Number(2.0));
        obj.map.insert(StringCow::from("c"), Amf0Value::Number(3.0));

        let ret_new = obj.shift_insert(1, StringCow::from("x"), Amf0Value::Number(9.0));
        assert!(ret_new.is_none(), "Inserting a brand‐new key should return None");

        let keys_after_insert: Vec<&str> = obj.map.keys().map(|k| k.as_ref()).collect();
        assert_eq!(
            keys_after_insert,
            vec!["a", "x", "b", "c"],
            "After shifting in 'x' at index 1, order should be [\"a\", \"x\", \"b\", \"c\"]"
        );

        let old_value = obj.shift_insert(0, StringCow::from("b"), Amf0Value::Number(7.0));
        assert_eq!(
            old_value,
            Some(Amf0Value::Number(2.0)),
            "Reinserting 'b' must return its previous value"
        );

        let keys_after_reinsert: Vec<&str> = obj.map.keys().map(|k| k.as_ref()).collect();
        assert_eq!(
            keys_after_reinsert,
            vec!["b", "a", "x", "c"],
            "After shifting 'b' to index 0, order should be [\"b\", \"a\", \"x\", \"c\"]"
        );

        assert_eq!(
            obj.map.get("b"),
            Some(&Amf0Value::Number(7.0)),
            "The value for key 'b' should now be updated to 7.0"
        );
    }

    #[test]
    #[cfg(not(feature = "preserve_order"))]
    fn test_remove_non_preserve_order() {
        let mut obj = Amf0Object::new();

        obj.map.insert(StringCow::from("key1"), Amf0Value::Number(10.0));
        obj.map
            .insert(StringCow::from("key2"), Amf0Value::String(StringCow::from("value")));
        obj.map.insert(StringCow::from("key3"), Amf0Value::Boolean(false));

        let removed = obj.remove("key2");
        assert_eq!(
            removed,
            Some(Amf0Value::String(StringCow::from("value"))),
            "remove should return the value for an existing key"
        );

        assert!(!obj.map.contains_key("key2"), "key2 should no longer exist after removal");
        assert!(
            obj.remove("missing").is_none(),
            "remove on a nonexistent key should return None"
        );
    }

    #[test]
    #[cfg(feature = "preserve_order")]
    fn test_remove_preserve_order_swap_remove_behavior() {
        let mut obj = Amf0Object::new();

        obj.map.insert(StringCow::from("a"), Amf0Value::Number(1.0));
        obj.map.insert(StringCow::from("b"), Amf0Value::Number(2.0));
        obj.map.insert(StringCow::from("c"), Amf0Value::Number(3.0));

        let removed = obj.remove("b");
        assert_eq!(
            removed,
            Some(Amf0Value::Number(2.0)),
            "remove should return the old value for key 'b'"
        );

        let keys_after: Vec<&str> = obj.map.keys().map(|k| k.as_ref()).collect();
        assert_eq!(
            keys_after,
            vec!["a", "c"],
            "After removing 'b', the order should be [\"a\", \"c\"]"
        );
        assert!(!obj.map.contains_key("b"), "key 'b' should be gone after removal");
        assert!(obj.map.contains_key("c"), "key 'c' should still be present");
        assert!(
            obj.remove("missing").is_none(),
            "remove on a nonexistent key should return None"
        );
    }

    #[test]
    #[cfg(not(feature = "preserve_order"))]
    fn test_remove_entry_non_preserve_order() {
        let mut obj = Amf0Object::new();

        obj.map.insert(StringCow::from("k1"), Amf0Value::Number(1.0));
        obj.map
            .insert(StringCow::from("k2"), Amf0Value::String(StringCow::from("v2")));
        obj.map.insert(StringCow::from("k3"), Amf0Value::Boolean(true));

        let entry = obj.remove_entry("k2");
        assert_eq!(
            entry,
            Some((StringCow::from("k2"), Amf0Value::String(StringCow::from("v2")))),
            "Expected remove_entry to return the removed key and value"
        );

        assert!(!obj.map.contains_key("k2"), "Key 'k2' should be gone after remove_entry");
        assert!(
            obj.remove_entry("missing").is_none(),
            "remove_entry on nonexistent key should return None"
        );
    }

    #[test]
    #[cfg(feature = "preserve_order")]
    fn test_remove_entry_preserve_order_swap_remove_entry() {
        let mut obj = Amf0Object::new();

        obj.map.insert(StringCow::from("a"), Amf0Value::Number(10.0));
        obj.map.insert(StringCow::from("b"), Amf0Value::Number(20.0));
        obj.map.insert(StringCow::from("c"), Amf0Value::Number(30.0));
        obj.map.insert(StringCow::from("d"), Amf0Value::Number(40.0));

        let entry = obj.remove_entry("b");
        assert_eq!(
            entry,
            Some((StringCow::from("b"), Amf0Value::Number(20.0))),
            "Expected remove_entry to return the removed key 'b' and its value"
        );

        let keys_after: Vec<&str> = obj.map.keys().map(|k| k.as_ref()).collect();
        assert_eq!(
            keys_after,
            vec!["a", "d", "c"],
            "After swap_remove_entry on 'b', order should be [\"a\", \"d\", \"c\"]"
        );
        assert!(!obj.map.contains_key("b"), "Key 'b' should be gone after remove_entry");
        assert!(obj.map.contains_key("d"), "Key 'd' should still be present");
        assert!(obj.map.contains_key("c"), "Key 'c' should still be present");
        assert!(
            obj.remove_entry("missing").is_none(),
            "remove_entry on nonexistent key should return None"
        );
    }

    #[test]
    #[cfg(feature = "preserve_order")]
    fn test_swap_remove_existing_and_order() {
        let mut obj = Amf0Object::new();

        obj.map.insert(StringCow::from("one"), Amf0Value::Number(1.0));
        obj.map.insert(StringCow::from("two"), Amf0Value::Number(2.0));
        obj.map.insert(StringCow::from("three"), Amf0Value::Number(3.0));
        obj.map.insert(StringCow::from("four"), Amf0Value::Number(4.0));

        let removed = obj.swap_remove("two");
        assert_eq!(
            removed,
            Some(Amf0Value::Number(2.0)),
            "swap_remove should return the removed value for key 'two'"
        );

        let keys_after: Vec<&str> = obj.map.keys().map(|k| k.as_ref()).collect();
        assert_eq!(
            keys_after,
            vec!["one", "four", "three"],
            "After swap_remove on 'two', order should be [\"one\", \"four\", \"three\"]"
        );
        assert!(!obj.map.contains_key("two"), "Key 'two' should no longer exist");
        assert!(obj.map.contains_key("four"), "Key 'four' should still be present");
        assert!(
            obj.swap_remove("absent").is_none(),
            "swap_remove on nonexistent key should return None"
        );
    }

    #[test]
    #[cfg(feature = "preserve_order")]
    fn test_swap_remove_entry_preserve_order() {
        let mut obj = Amf0Object::new();

        obj.map.insert(StringCow::from("a"), Amf0Value::Number(10.0));
        obj.map.insert(StringCow::from("b"), Amf0Value::Number(20.0));
        obj.map.insert(StringCow::from("c"), Amf0Value::Number(30.0));
        obj.map.insert(StringCow::from("d"), Amf0Value::Number(40.0));

        let entry = obj.swap_remove_entry("b");
        assert_eq!(
            entry,
            Some((StringCow::from("b"), Amf0Value::Number(20.0))),
            "Expected swap_remove_entry to return the removed key 'b' and its value"
        );

        let keys_after: Vec<&str> = obj.map.keys().map(|k| k.as_ref()).collect();
        assert_eq!(
            keys_after,
            vec!["a", "d", "c"],
            "After swap_remove_entry on 'b', order should be [\"a\", \"d\", \"c\"]"
        );

        assert!(!obj.map.contains_key("b"), "Key 'b' should no longer exist");
        assert!(obj.map.contains_key("d"), "Key 'd' should still be present");
        assert!(obj.map.contains_key("c"), "Key 'c' should still be present");
        assert!(
            obj.swap_remove_entry("missing").is_none(),
            "swap_remove_entry on nonexistent key should return None"
        );
    }

    #[test]
    #[cfg(feature = "preserve_order")]
    fn test_shift_remove_preserve_order() {
        let mut obj = Amf0Object::new();

        obj.map.insert(StringCow::from("a"), Amf0Value::Number(1.0));
        obj.map.insert(StringCow::from("b"), Amf0Value::Number(2.0));
        obj.map.insert(StringCow::from("c"), Amf0Value::Number(3.0));
        obj.map.insert(StringCow::from("d"), Amf0Value::Number(4.0));

        let removed = obj.shift_remove("b");
        assert_eq!(
            removed,
            Some(Amf0Value::Number(2.0)),
            "shift_remove should return the removed value for key 'b'"
        );

        let keys_after: Vec<&str> = obj.map.keys().map(|k| k.as_ref()).collect();
        assert_eq!(
            keys_after,
            vec!["a", "c", "d"],
            "After shift_remove on 'b', order should be [\"a\", \"c\", \"d\"]"
        );
        assert!(!obj.map.contains_key("b"), "Key 'b' should no longer exist");
        assert!(obj.map.contains_key("c"), "Key 'c' should still be present");
        assert!(obj.map.contains_key("d"), "Key 'd' should still be present");
        assert!(
            obj.shift_remove("missing").is_none(),
            "shift_remove on nonexistent key should return None"
        );
    }

    #[test]
    #[cfg(feature = "preserve_order")]
    fn test_shift_remove_entry_preserve_order() {
        let mut obj = Amf0Object::new();

        obj.map.insert(StringCow::from("a"), Amf0Value::Number(1.0));
        obj.map.insert(StringCow::from("b"), Amf0Value::Number(2.0));
        obj.map.insert(StringCow::from("c"), Amf0Value::Number(3.0));
        obj.map.insert(StringCow::from("d"), Amf0Value::Number(4.0));

        let entry = obj.shift_remove_entry("b");
        assert_eq!(
            entry,
            Some((StringCow::from("b"), Amf0Value::Number(2.0))),
            "Expected shift_remove_entry to return removed key 'b' and its value"
        );

        let keys_after: Vec<&str> = obj.map.keys().map(|k| k.as_ref()).collect();
        assert_eq!(
            keys_after,
            vec!["a", "c", "d"],
            "After shift_remove_entry on 'b', order should be [\"a\", \"c\", \"d\"]"
        );
        assert!(!obj.map.contains_key("b"), "Key 'b' should no longer exist");
        assert!(obj.map.contains_key("c"), "Key 'c' should still be present");
        assert!(obj.map.contains_key("d"), "Key 'd' should still be present");
        assert!(
            obj.shift_remove_entry("missing").is_none(),
            "shift_remove_entry on nonexistent key should return None"
        );
    }

    #[test]
    #[cfg(not(feature = "preserve_order"))]
    fn test_append_non_preserve_order_merges_and_clears_other() {
        let mut obj1 = Amf0Object::new();
        let mut obj2 = Amf0Object::new();

        obj1.map.insert(StringCow::from("k1"), Amf0Value::Number(1.0));
        obj1.map.insert(StringCow::from("k2"), Amf0Value::Number(2.0));

        obj2.map.insert(StringCow::from("k2"), Amf0Value::Number(22.0)); // collision
        obj2.map.insert(StringCow::from("k3"), Amf0Value::Number(3.0));
        obj1.append(&mut obj2);

        assert_eq!(obj1.map.len(), 3);
        assert_eq!(obj1.map.get("k1"), Some(&Amf0Value::Number(1.0)));
        assert_eq!(obj1.map.get("k2"), Some(&Amf0Value::Number(22.0)));
        assert_eq!(obj1.map.get("k3"), Some(&Amf0Value::Number(3.0)));
        assert!(obj2.map.is_empty(), "obj2 should be emptied after append");
    }

    #[test]
    #[cfg(feature = "preserve_order")]
    fn test_append_preserve_order_extends_and_clears_other() {
        let mut obj1 = Amf0Object::new();
        let mut obj2 = Amf0Object::new();

        obj1.map.insert(StringCow::from("a"), Amf0Value::Number(10.0));
        obj1.map.insert(StringCow::from("b"), Amf0Value::Number(20.0));

        obj2.map.insert(StringCow::from("b"), Amf0Value::Number(200.0)); // collision
        obj2.map.insert(StringCow::from("c"), Amf0Value::Number(30.0));
        obj1.append(&mut obj2);

        let keys_after: Vec<&str> = obj1.map.keys().map(|k| k.as_ref()).collect();
        let values_after: Vec<&Amf0Value> = obj1.map.values().collect();

        assert_eq!(
            keys_after,
            vec!["a", "b", "c"],
            "Order should preserve original keys, with duplicates replaced, then obj2's new keys"
        );

        assert_eq!(
            values_after,
            vec![&Amf0Value::Number(10.0), &Amf0Value::Number(200.0), &Amf0Value::Number(30.0),]
        );

        assert!(obj2.map.is_empty(), "obj2 should be emptied after append");
    }

    #[test]
    #[cfg(not(feature = "preserve_order"))]
    fn test_entry_vacant_inserts_value_non_preserve_order() {
        let mut obj = Amf0Object::new();

        match obj.entry("new_key") {
            Entry::Vacant(mut vacant) => {
                vacant.insert(Amf0Value::Number(42.0));
            }
            Entry::Occupied(_) => panic!("Expected Vacant for a missing key"),
        }

        assert_eq!(
            obj.map.get("new_key"),
            Some(&Amf0Value::Number(42.0)),
            "Value inserted via entry should be retrievable"
        );
    }

    #[test]
    #[cfg(feature = "preserve_order")]
    fn test_entry_vacant_inserts_value_preserve_order() {
        let mut obj = Amf0Object::new();
        match obj.entry("vacant_key") {
            Entry::Vacant(vacant) => {
                vacant.insert(Amf0Value::Number(123.0));
            }
            Entry::Occupied(_) => panic!("Expected Vacant for a missing key"),
        }

        let keys: Vec<&str> = obj.map.keys().map(|k| k.as_ref()).collect();
        assert_eq!(keys, vec!["vacant_key"]);
        assert_eq!(obj.map.get("vacant_key"), Some(&Amf0Value::Number(123.0)));
    }

    #[test]
    #[cfg(feature = "preserve_order")]
    fn test_entry_occupied_modifies_existing_value_preserve_order() {
        let mut obj = Amf0Object::new();
        obj.map.insert(StringCow::from("existing"), Amf0Value::Number(1.0));

        match obj.entry("existing") {
            Entry::Vacant(_) => panic!("Expected Occupied for an existing key"),
            Entry::Occupied(mut occupied) => {
                *occupied.get_mut() = Amf0Value::Number(99.0);
            }
        }

        assert_eq!(
            obj.map.get("existing"),
            Some(&Amf0Value::Number(99.0)),
            "Occupied entry modification should update the stored value"
        );
    }

    #[test]
    #[cfg(not(feature = "preserve_order"))]
    fn test_is_empty_non_preserve_order() {
        let mut obj = Amf0Object::new();
        assert!(obj.is_empty(), "Newly created Amf0Object should be empty");

        obj.map.insert(StringCow::from("key"), Amf0Value::Number(5.0));
        assert!(!obj.is_empty(), "Amf0Object should not be empty after insertion");

        obj.clear();
        assert!(obj.is_empty(), "Amf0Object should be empty after clear");
    }

    #[test]
    #[cfg(feature = "preserve_order")]
    fn test_is_empty_preserve_order() {
        let mut obj = Amf0Object::new();
        assert!(obj.is_empty(), "Newly created Amf0Object should be empty");

        obj.map.insert(StringCow::from("alpha"), Amf0Value::Number(1.0));
        assert!(!obj.is_empty(), "Amf0Object should not be empty after insertion");

        obj.shift_remove("alpha");
        assert!(obj.is_empty(), "Amf0Object should be empty after removing the only key");
    }

    #[test]
    #[cfg(not(feature = "preserve_order"))]
    fn test_iter_mut_non_preserve_order() {
        let mut obj = Amf0Object::new();
        obj.map.insert(StringCow::from("k1"), Amf0Value::Number(1.0));
        obj.map.insert(StringCow::from("k2"), Amf0Value::Number(2.0));
        obj.map.insert(StringCow::from("k3"), Amf0Value::Number(3.0));

        for (_k, v) in obj.iter_mut() {
            if let Amf0Value::Number(ref mut n) = *v {
                *n += 10.0;
            }
        }

        assert_eq!(obj.map.get("k1"), Some(&Amf0Value::Number(11.0)));
        assert_eq!(obj.map.get("k2"), Some(&Amf0Value::Number(12.0)));
        assert_eq!(obj.map.get("k3"), Some(&Amf0Value::Number(13.0)));
    }

    #[test]
    #[cfg(feature = "preserve_order")]
    fn test_iter_mut_preserve_order() {
        let mut obj = Amf0Object::new();
        obj.map.insert(StringCow::from("a"), Amf0Value::Number(5.0));
        obj.map.insert(StringCow::from("b"), Amf0Value::Number(6.0));
        obj.map.insert(StringCow::from("c"), Amf0Value::Number(7.0));

        let keys: Vec<&str> = obj.iter_mut().map(|(k, _)| k.as_ref()).collect();
        assert_eq!(keys, vec!["a", "b", "c"], "Iteration order should follow insertion order");

        for (_k, v) in obj.iter_mut() {
            *v = Amf0Value::Number(0.0);
        }

        assert_eq!(obj.map.get("a"), Some(&Amf0Value::Number(0.0)));
        assert_eq!(obj.map.get("b"), Some(&Amf0Value::Number(0.0)));
        assert_eq!(obj.map.get("c"), Some(&Amf0Value::Number(0.0)));
    }

    #[test]
    #[cfg(not(feature = "preserve_order"))]
    fn test_keys_non_preserve_order_sorted() {
        let mut obj = Amf0Object::new();

        obj.map.insert(StringCow::from("b"), Amf0Value::Number(2.0));
        obj.map.insert(StringCow::from("a"), Amf0Value::Number(1.0));
        obj.map.insert(StringCow::from("c"), Amf0Value::Number(3.0));

        let keys: Vec<&str> = obj.keys().map(|k| k.as_ref()).collect();
        assert_eq!(keys, vec!["a", "b", "c"], "Expected keys sorted lexicographically");
    }

    #[test]
    #[cfg(feature = "preserve_order")]
    fn test_keys_preserve_order_insertion() {
        let mut obj = Amf0Object::new();

        obj.map.insert(StringCow::from("b"), Amf0Value::Number(2.0));
        obj.map.insert(StringCow::from("a"), Amf0Value::Number(1.0));
        obj.map.insert(StringCow::from("c"), Amf0Value::Number(3.0));

        let keys: Vec<&str> = obj.keys().map(|k| k.as_ref()).collect();
        assert_eq!(keys, vec!["b", "a", "c"], "Expected keys in insertion order");
    }

    #[test]
    #[cfg(not(feature = "preserve_order"))]
    fn test_values_non_preserve_order_sorted_by_key() {
        let mut obj = Amf0Object::new();

        obj.map.insert(StringCow::from("b"), Amf0Value::Number(2.0));
        obj.map.insert(StringCow::from("a"), Amf0Value::Number(1.0));
        obj.map.insert(StringCow::from("c"), Amf0Value::Number(3.0));

        let values: Vec<&Amf0Value> = obj.values().collect();
        assert_eq!(
            values,
            vec![&Amf0Value::Number(1.0), &Amf0Value::Number(2.0), &Amf0Value::Number(3.0)],
            "Expected values in order of sorted keys"
        );
    }

    #[test]
    #[cfg(feature = "preserve_order")]
    fn test_values_preserve_order_insertion() {
        let mut obj = Amf0Object::new();

        obj.map.insert(StringCow::from("b"), Amf0Value::Number(2.0));
        obj.map.insert(StringCow::from("a"), Amf0Value::Number(1.0));
        obj.map.insert(StringCow::from("c"), Amf0Value::Number(3.0));

        let values: Vec<&Amf0Value> = obj.values().collect();
        assert_eq!(
            values,
            vec![&Amf0Value::Number(2.0), &Amf0Value::Number(1.0), &Amf0Value::Number(3.0)],
            "Expected values in insertion order"
        );
    }

    #[test]
    #[cfg(not(feature = "preserve_order"))]
    fn test_values_mut_non_preserve_order() {
        let mut obj = Amf0Object::new();

        obj.map.insert(StringCow::from("b"), Amf0Value::Number(2.0));
        obj.map.insert(StringCow::from("a"), Amf0Value::Number(1.0));
        obj.map.insert(StringCow::from("c"), Amf0Value::Number(3.0));

        for value in obj.values_mut() {
            if let Amf0Value::Number(ref mut n) = *value {
                *n *= 10.0;
            }
        }

        let sorted_keys: Vec<&str> = obj.keys().map(|k| k.as_ref()).collect();
        assert_eq!(sorted_keys, vec!["a", "b", "c"]);

        assert_eq!(obj.map.get("a"), Some(&Amf0Value::Number(10.0)));
        assert_eq!(obj.map.get("b"), Some(&Amf0Value::Number(20.0)));
        assert_eq!(obj.map.get("c"), Some(&Amf0Value::Number(30.0)));
    }

    #[test]
    #[cfg(feature = "preserve_order")]
    fn test_values_mut_preserve_order() {
        let mut obj = Amf0Object::new();

        obj.map.insert(StringCow::from("b"), Amf0Value::Number(2.0));
        obj.map.insert(StringCow::from("a"), Amf0Value::Number(1.0));
        obj.map.insert(StringCow::from("c"), Amf0Value::Number(3.0));

        for value in obj.values_mut() {
            *value = Amf0Value::Number(0.0);
        }

        let keys_in_order: Vec<&str> = obj.keys().map(|k| k.as_ref()).collect();
        assert_eq!(keys_in_order, vec!["b", "a", "c"]);

        assert_eq!(obj.map.get("b"), Some(&Amf0Value::Number(0.0)));
        assert_eq!(obj.map.get("a"), Some(&Amf0Value::Number(0.0)));
        assert_eq!(obj.map.get("c"), Some(&Amf0Value::Number(0.0)));
    }

    #[test]
    #[cfg(not(feature = "preserve_order"))]
    fn test_into_values_consume_and_return_sorted_values() {
        let mut obj = Amf0Object::new();

        obj.map.insert(StringCow::from("b"), Amf0Value::Number(2.0));
        obj.map.insert(StringCow::from("a"), Amf0Value::Number(1.0));
        obj.map.insert(StringCow::from("c"), Amf0Value::Number(3.0));

        let values: Vec<Amf0Value> = obj.into_values().collect();
        assert_eq!(
            values,
            vec![Amf0Value::Number(1.0), Amf0Value::Number(2.0), Amf0Value::Number(3.0)],
            "into_values should yield values in key‐sorted order for BTreeMap"
        );
    }

    #[test]
    #[cfg(feature = "preserve_order")]
    fn test_into_values_consume_and_return_insertion_order_values() {
        let mut obj = Amf0Object::new();

        obj.map.insert(StringCow::from("b"), Amf0Value::Number(2.0));
        obj.map.insert(StringCow::from("a"), Amf0Value::Number(1.0));
        obj.map.insert(StringCow::from("c"), Amf0Value::Number(3.0));

        let values: Vec<Amf0Value> = obj.into_values().collect();
        assert_eq!(
            values,
            vec![Amf0Value::Number(2.0), Amf0Value::Number(1.0), Amf0Value::Number(3.0)],
            "into_values should yield values in insertion order for IndexMap"
        );
    }

    #[test]
    #[cfg(not(feature = "preserve_order"))]
    fn test_retain_non_preserve_order() {
        let mut obj = Amf0Object::new();

        obj.map.insert(StringCow::from("a"), Amf0Value::Number(1.0));
        obj.map.insert(StringCow::from("b"), Amf0Value::Number(2.0));
        obj.map.insert(StringCow::from("c"), Amf0Value::Number(3.0));
        obj.map.insert(StringCow::from("d"), Amf0Value::Number(4.0));

        obj.retain(|_key, value| if let Amf0Value::Number(n) = *value { n > 2.0 } else { false });

        let remaining_keys: Vec<&str> = obj.keys().map(|k| k.as_ref()).collect();
        assert_eq!(remaining_keys, vec!["c", "d"]);

        assert_eq!(obj.map.get("c"), Some(&Amf0Value::Number(3.0)));
        assert_eq!(obj.map.get("d"), Some(&Amf0Value::Number(4.0)));

        assert!(!obj.map.contains_key("a"));
        assert!(!obj.map.contains_key("b"));
    }

    #[test]
    #[cfg(feature = "preserve_order")]
    fn test_retain_preserve_order() {
        let mut obj = Amf0Object::new();

        obj.map.insert(StringCow::from("x1"), Amf0Value::Number(1.0));
        obj.map.insert(StringCow::from("x2"), Amf0Value::Number(2.0));
        obj.map.insert(StringCow::from("x3"), Amf0Value::Number(3.0));
        obj.map.insert(StringCow::from("x4"), Amf0Value::Number(4.0));

        obj.retain(|_key, value| {
            if let Amf0Value::Number(n) = *value {
                (n as i64) % 2 == 0
            } else {
                false
            }
        });

        let remaining_keys: Vec<&str> = obj.keys().map(|k| k.as_ref()).collect();
        assert_eq!(remaining_keys, vec!["x2", "x4"]);

        assert_eq!(obj.map.get("x2"), Some(&Amf0Value::Number(2.0)));
        assert_eq!(obj.map.get("x4"), Some(&Amf0Value::Number(4.0)));

        assert!(!obj.map.contains_key("x1"));
        assert!(!obj.map.contains_key("x3"));
    }

    #[test]
    #[cfg(feature = "preserve_order")]
    fn test_sort_keys_preserve_order() {
        let mut obj = Amf0Object::new();

        obj.map.insert(StringCow::from("b"), Amf0Value::Number(2.0));
        obj.map.insert(StringCow::from("d"), Amf0Value::Number(4.0));
        obj.map.insert(StringCow::from("a"), Amf0Value::Number(1.0));
        obj.map.insert(StringCow::from("c"), Amf0Value::Number(3.0));

        let before: Vec<&str> = obj.keys().map(|k| k.as_ref()).collect();
        assert_eq!(before, vec!["b", "d", "a", "c"]);

        obj.sort_keys();

        let after: Vec<&str> = obj.keys().map(|k| k.as_ref()).collect();
        assert_eq!(after, vec!["a", "b", "c", "d"]);

        assert_eq!(obj.map.get("a"), Some(&Amf0Value::Number(1.0)));
        assert_eq!(obj.map.get("b"), Some(&Amf0Value::Number(2.0)));
        assert_eq!(obj.map.get("c"), Some(&Amf0Value::Number(3.0)));
        assert_eq!(obj.map.get("d"), Some(&Amf0Value::Number(4.0)));
    }

    #[test]
    fn test_index_returns_value_for_existing_key() {
        let mut obj = Amf0Object::new();
        obj.map.insert(StringCow::from("alpha"), Amf0Value::Number(3.21));

        let v: &Amf0Value = &obj["alpha"];
        assert_eq!(v, &Amf0Value::Number(3.21));
    }

    #[test]
    #[should_panic]
    fn test_index_panics_on_missing_key() {
        let obj = Amf0Object::new();
        // Indexing a non‐existent key should panic
        let _ = &obj["nonexistent"];
    }

    #[test]
    fn test_index_mut_modifies_existing_value() {
        let mut obj = Amf0Object::new();
        obj.map.insert(StringCow::from("key1"), Amf0Value::Number(5.0));

        obj["key1"] = Amf0Value::Number(42.0);

        assert_eq!(
            obj.map.get("key1"),
            Some(&Amf0Value::Number(42.0)),
            "IndexMut should allow updating the value"
        );
    }

    #[test]
    #[should_panic(expected = "no entry found for key")]
    fn test_index_mut_panics_on_missing_key() {
        let mut obj = Amf0Object::new();
        obj["nonexistent"] = Amf0Value::Boolean(true);
    }

    #[cfg(not(feature = "preserve_order"))]
    #[test]
    fn test_debug_formats_sorted_map_non_preserve_order() {
        let mut obj = Amf0Object::new();

        obj.map.insert(StringCow::from("b"), Amf0Value::Number(2.0));
        obj.map.insert(StringCow::from("a"), Amf0Value::Number(1.0));

        insta::assert_debug_snapshot!(obj, @r#"
        {
            Ref(
                "b",
            ): Number(
                2.0,
            ),
            Ref(
                "a",
            ): Number(
                1.0,
            ),
        }
        "#);
    }

    #[cfg(feature = "preserve_order")]
    #[test]
    fn test_debug_formats_insertion_order_preserve_order() {
        let mut obj = Amf0Object::new();

        obj.map.insert(StringCow::from("b"), Amf0Value::Number(2.0));
        obj.map.insert(StringCow::from("a"), Amf0Value::Number(1.0));

        insta::assert_debug_snapshot!(obj, @r#"
        {
            Ref(
                "b",
            ): Number(
                2.0,
            ),
            Ref(
                "a",
            ): Number(
                1.0,
            ),
        }
        "#);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_visit_unit_returns_empty_object() {
        use serde::de::value::{Error as DeError, UnitDeserializer};

        // Using Serde's built-in UnitDeserializer, which directly calls visit_unit()
        let deser: UnitDeserializer<DeError> = UnitDeserializer::new();
        let obj = Amf0Object::deserialize(deser).unwrap();

        assert!(obj.is_empty(), "visit_unit should produce an empty Amf0Object");
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_expecting_error_message() {
        use serde::de::value::Error as DeError;
        use serde::de::{self};

        struct WrongTypeDeserializer;

        impl<'de> de::Deserializer<'de> for WrongTypeDeserializer {
            type Error = DeError;

            serde::forward_to_deserialize_any! {
                bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes byte_buf
                option unit unit_struct newtype_struct seq tuple tuple_struct map struct
                enum identifier ignored_any
            }

            fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
            where
                V: de::Visitor<'de>,
            {
                Err(de::Error::invalid_type(de::Unexpected::Bool(true), &visitor))
            }
        }

        let err = Amf0Object::deserialize(WrongTypeDeserializer).unwrap_err();
        let msg = err.to_string();

        assert!(
            msg.contains("a map"),
            "expecting() should include 'a map' in the error message, got: '{}'",
            msg
        );
    }

    #[cfg(not(feature = "preserve_order"))]
    #[test]
    fn test_extend_non_preserve_order_inserts_and_sorts() {
        let mut obj = Amf0Object::new();

        let items = vec![
            (StringCow::from("b"), Amf0Value::Number(2.0)),
            (StringCow::from("a"), Amf0Value::Number(1.0)),
            (StringCow::from("a"), Amf0Value::Number(5.0)),
            (StringCow::from("c"), Amf0Value::Number(3.0)),
        ];

        obj.extend(items);

        let keys: Vec<&str> = obj.keys().map(|k| k.as_ref()).collect();
        assert_eq!(
            keys,
            vec!["a", "b", "c"],
            "Non-preserve_order: keys should be sorted lexicographically"
        );

        assert_eq!(obj.map.get("a"), Some(&Amf0Value::Number(5.0)));
        assert_eq!(obj.map.get("b"), Some(&Amf0Value::Number(2.0)));
        assert_eq!(obj.map.get("c"), Some(&Amf0Value::Number(3.0)));
    }

    #[cfg(feature = "preserve_order")]
    #[test]
    fn test_extend_preserve_order_inserts_and_overwrites() {
        let mut obj = Amf0Object::new();

        obj.map.insert(StringCow::from("a"), Amf0Value::Number(1.0));

        let items = vec![
            (StringCow::from("a"), Amf0Value::Number(5.0)),
            (StringCow::from("b"), Amf0Value::Number(2.0)),
            (StringCow::from("a"), Amf0Value::Number(7.0)),
            (StringCow::from("c"), Amf0Value::Number(3.0)),
        ];

        obj.extend(items);

        let keys: Vec<&str> = obj.keys().map(|k| k.as_ref()).collect();
        assert_eq!(
            keys,
            vec!["a", "b", "c"],
            "Preserve_order: key 'a' remains at front, then 'b', then 'c'"
        );

        assert_eq!(obj.map.get("a"), Some(&Amf0Value::Number(7.0)));
        assert_eq!(obj.map.get("b"), Some(&Amf0Value::Number(2.0)));
        assert_eq!(obj.map.get("c"), Some(&Amf0Value::Number(3.0)));
    }

    #[cfg(not(feature = "preserve_order"))]
    #[test]
    fn test_keys_iterator_next_back_and_len_non_preserve_order() {
        let mut obj = Amf0Object::new();

        obj.map.insert(StringCow::from("b"), Amf0Value::Number(2.0));
        obj.map.insert(StringCow::from("a"), Amf0Value::Number(1.0));
        obj.map.insert(StringCow::from("c"), Amf0Value::Number(3.0));

        let mut keys_iter = obj.keys();

        assert_eq!(keys_iter.len(), 3);

        let last = keys_iter.next_back().expect("Expected a last key");
        assert_eq!(last.as_ref(), "c");

        assert_eq!(keys_iter.len(), 2);

        let first = keys_iter.next().expect("Expected a first key");
        assert_eq!(first.as_ref(), "a");

        assert_eq!(keys_iter.len(), 1);
    }

    #[cfg(feature = "preserve_order")]
    #[test]
    fn test_keys_iterator_next_back_and_len_preserve_order() {
        let mut obj = Amf0Object::new();

        obj.map.insert(StringCow::from("b"), Amf0Value::Number(2.0));
        obj.map.insert(StringCow::from("a"), Amf0Value::Number(1.0));
        obj.map.insert(StringCow::from("c"), Amf0Value::Number(3.0));

        let mut keys_iter = obj.keys();

        assert_eq!(keys_iter.len(), 3);

        let last = keys_iter.next_back().expect("Expected a last key");
        assert_eq!(last.as_ref(), "c");

        assert_eq!(keys_iter.len(), 2);

        let first = keys_iter.next().expect("Expected a first key");
        assert_eq!(first.as_ref(), "b");
        assert_eq!(keys_iter.len(), 1);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_into_deserializer_returns_self() {
        use serde::de::IntoDeserializer;

        use crate::Amf0Error;

        let obj = Amf0Object::new();

        let deser: &Amf0Object = <&Amf0Object as IntoDeserializer<Amf0Error>>::into_deserializer(&obj);
        assert!(
            std::ptr::eq(deser, &obj),
            "into_deserializer should return the exact same &Amf0Object reference"
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_into_deserializer_as_deserializer_errors_on_primitive() {
        use serde::Deserialize;
        use serde::de::IntoDeserializer;

        let obj = Amf0Object::new();
        let deser = obj.into_deserializer();

        let result = <bool as Deserialize>::deserialize(&deser);
        assert!(result.is_err(), "Deserializing a bool from an Amf0Object should error");
    }

    #[derive(Debug, Deserialize, PartialEq)]
    struct TestObject {
        string: String,
        number: u32,
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_deserialize_any_for_struct() {
        let mut obj = Amf0Object::new();
        obj.map
            .insert(StringCow::from("string"), Amf0Value::String(StringCow::from("apple")));
        obj.map.insert(StringCow::from("number"), Amf0Value::Number(30.0));

        let testobj: TestObject = Deserialize::deserialize(&obj).unwrap();
        assert_eq!(
            testobj,
            TestObject {
                string: "apple".into(),
                number: 30,
            }
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_deserialize_any_errors_on_missing_field() {
        let mut obj = Amf0Object::new();
        obj.map
            .insert(StringCow::from("string"), Amf0Value::String(StringCow::from("banana")));

        let result: Result<TestObject, _> = Deserialize::deserialize(&obj);
        assert!(result.is_err(), "Missing required field should cause an error");
    }

    #[test]
    fn test_entry_key_occupied() {
        let mut obj = Amf0Object::new();
        obj.map.insert(StringCow::from("serde"), Amf0Value::Number(42.0));
        let entry = obj.entry("serde");

        assert_eq!(entry.key(), &StringCow::from("serde"));
    }

    #[test]
    fn test_or_insert_vacant_and_occupied() {
        let mut obj = Amf0Object::new();

        let v = obj.entry("k").or_insert(Amf0Value::Number(10.0));
        assert_eq!(*v, Amf0Value::Number(10.0));
        assert_eq!(
            obj.map.get("k"),
            Some(&Amf0Value::Number(10.0)),
            "Value should be inserted for vacant entry"
        );

        let v2 = obj.entry("k").or_insert(Amf0Value::Number(20.0));
        assert_eq!(*v2, Amf0Value::Number(10.0));

        *v2 = Amf0Value::Number(30.0);
        assert_eq!(
            obj.map.get("k"),
            Some(&Amf0Value::Number(30.0)),
            "Value should be updated to 30.0 via the occupied branch"
        );
    }

    #[test]
    fn test_or_insert_with_vacant_and_occupied() {
        use std::cell::Cell;

        let mut obj = Amf0Object::new();

        let called = Cell::new(false);
        let default_closure = || {
            called.set(true);
            Amf0Value::Number(5.0)
        };

        let v = obj.entry("key1").or_insert_with(default_closure);
        assert!(called.get(), "Closure should have been called for vacant entry");
        assert_eq!(*v, Amf0Value::Number(5.0));
        assert_eq!(
            obj.map.get("key1"),
            Some(&Amf0Value::Number(5.0)),
            "Value for 'key1' should be inserted as 5.0"
        );

        called.set(false);
        let v2 = obj.entry("key1").or_insert_with(|| {
            called.set(true);
            Amf0Value::Number(10.0)
        });
        assert!(!called.get(), "Closure should not be called for occupied entry");
        assert_eq!(*v2, Amf0Value::Number(5.0));

        *v2 = Amf0Value::Number(7.0);
        assert_eq!(
            obj.map.get("key1"),
            Some(&Amf0Value::Number(7.0)),
            "Value for 'key1' should be updated to 7.0 via occupied branch"
        );
    }

    #[test]
    fn test_and_modify_on_occupied_entry() {
        let mut obj = Amf0Object::new();
        obj.map.insert(StringCow::from("key1"), Amf0Value::Number(1.0));

        obj.entry("key1").and_modify(|v| {
            if let Amf0Value::Number(n) = v {
                *n = 5.0;
            }
        });

        assert_eq!(
            obj.map.get("key1"),
            Some(&Amf0Value::Number(5.0)),
            "and_modify should change the existing value from 1.0 to 5.0"
        );
    }

    #[test]
    fn test_and_modify_on_vacant_entry() {
        let mut obj = Amf0Object::new();
        assert!(!obj.map.contains_key("missing"));

        obj.entry("missing").and_modify(|v| {
            if let Amf0Value::Number(n) = v {
                *n = 100.0;
            }
        });

        assert!(
            !obj.map.contains_key("missing"),
            "and_modify on a vacant entry should not insert anything"
        );
    }

    #[test]
    fn test_occupied_entry_key_and_get() {
        let mut obj = Amf0Object::new();

        obj.map.insert(StringCow::from("my_key"), Amf0Value::Number(123.0));

        match obj.entry("my_key") {
            Entry::Occupied(occupied) => {
                let key_ref: &StringCow = occupied.key();
                assert_eq!(key_ref.as_ref(), "my_key");

                let value_ref: &Amf0Value = occupied.get();
                assert_eq!(value_ref, &Amf0Value::Number(123.0));
            }
            Entry::Vacant(_) => panic!("Expected Occupied variant for an existing key"),
        }
    }

    #[test]
    fn test_occupied_entry_into_mut_and_insert() {
        let mut obj = Amf0Object::new();

        obj.map.insert(StringCow::from("key1"), Amf0Value::Number(10.0));

        if let Entry::Occupied(occ) = obj.entry("key1") {
            let value_ref: &mut Amf0Value = occ.into_mut();
            if let Amf0Value::Number(n) = value_ref {
                *n = 20.0;
            }
        } else {
            panic!("Expected Occupied variant");
        }

        assert_eq!(
            obj.map.get("key1"),
            Some(&Amf0Value::Number(20.0)),
            "into_mut should allow modifying the existing value"
        );

        if let Entry::Occupied(mut occ2) = obj.entry("key1") {
            let old = occ2.insert(Amf0Value::Number(30.0));
            assert_eq!(old, Amf0Value::Number(20.0), "insert should return the old value");
        } else {
            panic!("Expected Occupied variant for insert test");
        }

        assert_eq!(
            obj.map.get("key1"),
            Some(&Amf0Value::Number(30.0)),
            "insert should replace with the new value"
        );
    }

    #[cfg(not(feature = "preserve_order"))]
    #[test]
    fn test_occupied_entry_remove_non_preserve_order() {
        let mut obj = Amf0Object::new();

        obj.map.insert(StringCow::from("key1"), Amf0Value::Number(1.0));

        if let Entry::Occupied(occ) = obj.entry("key1") {
            let removed_value = occ.remove();
            assert_eq!(removed_value, Amf0Value::Number(1.0), "remove should return the old value");
        } else {
            panic!("Expected Occupied variant");
        }

        assert!(!obj.map.contains_key("key1"), "Key should be removed after calling remove()");
    }

    #[cfg(feature = "preserve_order")]
    #[test]
    fn test_occupied_entry_remove_preserve_order() {
        let mut obj = Amf0Object::new();

        obj.map.insert(StringCow::from("a"), Amf0Value::Number(10.0));
        obj.map.insert(StringCow::from("b"), Amf0Value::Number(20.0));
        obj.map.insert(StringCow::from("c"), Amf0Value::Number(30.0));

        if let Entry::Occupied(occ) = obj.entry("b") {
            let removed_value = occ.remove();
            assert_eq!(removed_value, Amf0Value::Number(20.0), "remove should return the old value");
        } else {
            panic!("Expected Occupied variant");
        }

        let keys_after: Vec<&str> = obj.keys().map(|k| k.as_ref()).collect();
        assert_eq!(
            keys_after,
            vec!["a", "c"],
            "After remove, key 'b' should be gone and 'c' should occupy its position"
        );
        assert!(!obj.map.contains_key("b"), "Key 'b' should be removed");
    }

    #[cfg(feature = "preserve_order")]
    #[test]
    fn test_occupied_entry_swap_remove_preserve_order() {
        let mut obj = Amf0Object::new();

        obj.map.insert(StringCow::from("x"), Amf0Value::Number(1.0));
        obj.map.insert(StringCow::from("y"), Amf0Value::Number(2.0));
        obj.map.insert(StringCow::from("z"), Amf0Value::Number(3.0));

        if let Entry::Occupied(occ) = obj.entry("y") {
            let removed = occ.swap_remove();
            assert_eq!(
                removed,
                Amf0Value::Number(2.0),
                "swap_remove should return the old value for 'y'"
            );
        } else {
            panic!("Expected Occupied variant for key 'y'");
        }

        let keys_after: Vec<&str> = obj.keys().map(|k| k.as_ref()).collect();
        assert_eq!(
            keys_after,
            vec!["x", "z"],
            "After swap_remove, 'y' should be gone and 'z' should occupy its slot"
        );
        assert!(!obj.map.contains_key("y"), "Key 'y' should be removed");
        assert!(obj.map.contains_key("z"), "Key 'z' should still be present");
    }

    #[cfg(feature = "preserve_order")]
    #[test]
    fn test_occupied_entry_shift_remove_preserve_order() {
        let mut obj = Amf0Object::new();

        obj.map.insert(StringCow::from("a"), Amf0Value::Number(1.0));
        obj.map.insert(StringCow::from("b"), Amf0Value::Number(2.0));
        obj.map.insert(StringCow::from("c"), Amf0Value::Number(3.0));
        obj.map.insert(StringCow::from("d"), Amf0Value::Number(4.0));

        if let Entry::Occupied(occ) = obj.entry("b") {
            let removed = occ.shift_remove();
            assert_eq!(
                removed,
                Amf0Value::Number(2.0),
                "shift_remove should return the old value for 'b'"
            );
        } else {
            panic!("Expected Occupied variant for key 'b'");
        }

        let keys_after: Vec<&str> = obj.keys().map(|k| k.as_ref()).collect();
        assert_eq!(
            keys_after,
            vec!["a", "c", "d"],
            "After shift_remove, 'b' should be gone and order should be [\"a\", \"c\", \"d\"]"
        );
        assert!(!obj.map.contains_key("b"), "Key 'b' should have been removed");
        assert!(obj.map.contains_key("c"), "Key 'c' should still be present");
        assert!(obj.map.contains_key("d"), "Key 'd' should still be present");
    }

    #[cfg(not(feature = "preserve_order"))]
    #[test]
    fn test_occupied_entry_remove_entry_non_preserve_order() {
        let mut obj = Amf0Object::new();

        obj.map.insert(StringCow::from("key1"), Amf0Value::Number(10.0));

        if let Entry::Occupied(occ) = obj.entry("key1") {
            let (removed_key, removed_value) = occ.remove_entry();
            assert_eq!(removed_key, StringCow::from("key1"));
            assert_eq!(removed_value, Amf0Value::Number(10.0));
        } else {
            panic!("Expected Occupied variant for 'key1'");
        }

        assert!(!obj.map.contains_key("key1"));
    }

    #[cfg(feature = "preserve_order")]
    #[test]
    fn test_occupied_entry_remove_entry_preserve_order() {
        let mut obj = Amf0Object::new();

        obj.map.insert(StringCow::from("a"), Amf0Value::Number(1.0));
        obj.map.insert(StringCow::from("b"), Amf0Value::Number(2.0));
        obj.map.insert(StringCow::from("c"), Amf0Value::Number(3.0));

        if let Entry::Occupied(occ) = obj.entry("b") {
            let (removed_key, removed_value) = occ.remove_entry();
            assert_eq!(removed_key, StringCow::from("b"));
            assert_eq!(removed_value, Amf0Value::Number(2.0));
        } else {
            panic!("Expected Occupied variant for 'b'");
        }

        let remaining_keys: Vec<&str> = obj.keys().map(|k| k.as_ref()).collect();
        assert_eq!(
            remaining_keys,
            vec!["a", "c"],
            "After remove_entry on 'b', keys should be [\"a\", \"c\"]"
        );
        assert!(!obj.map.contains_key("b"), "Key 'b' should be gone");
        assert!(obj.map.contains_key("c"), "Key 'c' should still be present");
    }

    #[cfg(feature = "preserve_order")]
    #[test]
    fn test_swap_remove_entry_preserve_order_occupied_entry() {
        let mut obj = Amf0Object::new();

        // Insert multiple entries in insertion order: ["a", "b", "c", "d"]
        obj.map.insert(StringCow::from("a"), Amf0Value::Number(1.0));
        obj.map.insert(StringCow::from("b"), Amf0Value::Number(2.0));
        obj.map.insert(StringCow::from("c"), Amf0Value::Number(3.0));
        obj.map.insert(StringCow::from("d"), Amf0Value::Number(4.0));

        // Remove "b" via OccupiedEntry::swap_remove_entry()
        if let Entry::Occupied(occ) = obj.entry("b") {
            let (removed_key, removed_value) = occ.swap_remove_entry();
            assert_eq!(removed_key, StringCow::from("b"));
            assert_eq!(removed_value, Amf0Value::Number(2.0));
        } else {
            panic!("Expected Occupied variant for key 'b'");
        }

        // After swap_remove_entry, "d" should replace "b"'s position; remaining keys: ["a", "d", "c"]
        let remaining_keys: Vec<&str> = obj.keys().map(|k| k.as_ref()).collect();
        assert_eq!(
            remaining_keys,
            vec!["a", "d", "c"],
            "After swap_remove_entry on 'b', keys should be [\"a\", \"d\", \"c\"]"
        );
        assert!(!obj.map.contains_key("b"), "Key 'b' should have been removed");
        assert!(obj.map.contains_key("d"), "Key 'd' should still be present");
        assert!(obj.map.contains_key("c"), "Key 'c' should still be present");
    }

    #[test]
    #[cfg(feature = "preserve_order")]
    fn test_occupied_entry_shift_remove_entry_preserve_order() {
        let mut obj = Amf0Object::new();

        obj.map.insert(StringCow::from("a"), Amf0Value::Number(1.0));
        obj.map.insert(StringCow::from("b"), Amf0Value::Number(2.0));
        obj.map.insert(StringCow::from("c"), Amf0Value::Number(3.0));
        obj.map.insert(StringCow::from("d"), Amf0Value::Number(4.0));

        if let Entry::Occupied(occ) = obj.entry("b") {
            let (removed_key, removed_value) = occ.shift_remove_entry();
            assert_eq!(
                removed_key,
                StringCow::from("b"),
                "shift_remove_entry should return the removed key 'b'"
            );
            assert_eq!(
                removed_value,
                Amf0Value::Number(2.0),
                "shift_remove_entry should return the correct value for 'b'"
            );
        } else {
            panic!("Expected Occupied variant for key 'b'");
        }

        let remaining_keys: Vec<&str> = obj.keys().map(|k| k.as_ref()).collect();
        assert_eq!(
            remaining_keys,
            vec!["a", "c", "d"],
            "After shift_remove_entry on 'b', keys should be [\"a\", \"c\", \"d\"]"
        );
        assert!(!obj.map.contains_key("b"), "Key 'b' should have been removed");
        assert!(obj.map.contains_key("c"), "Key 'c' should still be present");
        assert!(obj.map.contains_key("d"), "Key 'd' should still be present");
    }

    #[test]
    fn test_into_iterator_mutates_values_non_preserve_order() {
        let mut obj = Amf0Object::new();

        obj.map.insert(StringCow::from("b"), Amf0Value::Number(2.0));
        obj.map.insert(StringCow::from("a"), Amf0Value::Number(1.0));
        obj.map.insert(StringCow::from("c"), Amf0Value::Number(3.0));

        for (_, value) in &mut obj {
            if let Amf0Value::Number(n) = value {
                *n += 10.0;
            }
        }

        assert_eq!(obj.map.get("a"), Some(&Amf0Value::Number(11.0)));
        assert_eq!(obj.map.get("b"), Some(&Amf0Value::Number(12.0)));
        assert_eq!(obj.map.get("c"), Some(&Amf0Value::Number(13.0)));
    }

    #[test]
    #[cfg(feature = "preserve_order")]
    fn test_into_iterator_preserve_order_counts_and_mutates() {
        let mut obj = Amf0Object::new();

        obj.map.insert(StringCow::from("x"), Amf0Value::Number(5.0));
        obj.map.insert(StringCow::from("y"), Amf0Value::Number(6.0));
        obj.map.insert(StringCow::from("z"), Amf0Value::Number(7.0));

        let mut seen_keys = Vec::new();
        for (key, value) in &mut obj {
            seen_keys.push(key.as_ref().to_string());
            *value = Amf0Value::Number(0.0);
        }

        assert_eq!(seen_keys, vec!["x", "y", "z"]);
        assert_eq!(obj.map.get("x"), Some(&Amf0Value::Number(0.0)));
        assert_eq!(obj.map.get("y"), Some(&Amf0Value::Number(0.0)));
        assert_eq!(obj.map.get("z"), Some(&Amf0Value::Number(0.0)));
    }
}
