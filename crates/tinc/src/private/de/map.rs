use std::borrow::Borrow;
use std::collections::{BTreeMap, HashMap, btree_map, hash_map};
use std::hash::Hash;

pub(crate) trait Map<K, V> {
    fn get_ref<'a, Q>(&'a self, key: &Q) -> Option<&'a V>
    where
        K: Borrow<Q>,
        Q: ?Sized + Hash + Eq + Ord;

    fn get(&mut self, key: K) -> &mut V
    where
        V: Default;
    fn insert(&mut self, key: K, value: V) -> Option<MappableKey>
    where
        for<'a> MappableKey: From<&'a K>;
    fn reserve(&mut self, additional: usize);
}

impl<K: std::hash::Hash + std::cmp::Eq, V> Map<K, V> for HashMap<K, V> {
    #[inline]
    fn get_ref<'a, Q>(&'a self, key: &Q) -> Option<&'a V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + Ord + ?Sized,
    {
        HashMap::get(self, key.borrow())
    }

    #[inline]
    fn get(&mut self, key: K) -> &mut V
    where
        V: Default,
    {
        HashMap::entry(self, key).or_default()
    }

    fn insert(&mut self, key: K, value: V) -> Option<MappableKey>
    where
        for<'a> MappableKey: From<&'a K>,
    {
        match HashMap::entry(self, key) {
            hash_map::Entry::Occupied(entry) => Some(MappableKey::from(entry.key())),
            hash_map::Entry::Vacant(entry) => {
                entry.insert(value);
                None
            }
        }
    }

    #[inline]
    fn reserve(&mut self, additional: usize) {
        HashMap::reserve(self, additional);
    }
}

impl<K: std::cmp::Ord, V> Map<K, V> for BTreeMap<K, V> {
    #[inline]
    fn get_ref<'a, Q>(&'a self, key: &Q) -> Option<&'a V>
    where
        K: Borrow<Q>,
        Q: ?Sized + Hash + Eq + Ord,
    {
        BTreeMap::get(self, key.borrow())
    }

    #[inline]
    fn get(&mut self, key: K) -> &mut V
    where
        V: Default,
    {
        BTreeMap::entry(self, key).or_default()
    }

    fn insert(&mut self, key: K, value: V) -> Option<MappableKey>
    where
        for<'a> MappableKey: From<&'a K>,
    {
        match BTreeMap::entry(self, key) {
            btree_map::Entry::Occupied(entry) => Some(MappableKey::from(entry.key())),
            btree_map::Entry::Vacant(entry) => {
                entry.insert(value);
                None
            }
        }
    }

    #[inline]
    fn reserve(&mut self, _: usize) {}
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum MappableKey {
    Str(Box<str>),
    Int(i64),
    UInt(u64),
    Bool(bool),
}

impl std::fmt::Display for MappableKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Str(s) => s.fmt(f),
            Self::Int(i) => i.fmt(f),
            Self::UInt(u) => u.fmt(f),
            Self::Bool(b) => b.fmt(f),
        }
    }
}

macro_rules! impl_from_for_mappable_key {
    ($variant:ident: $($t:ty),*) => {
        $(
            impl From<$t> for MappableKey {
                #[inline]
                fn from(value: $t) -> Self {
                    Self::$variant(value.clone().into())
                }
            }
        )*
    }
}

trait IntoBorrowed<T> {
    type Borrowed: ?Sized;

    fn to_borrow(&self) -> &Self::Borrowed;
}

trait MapKeyBack: Sized {
    type Value<'a>;

    fn to_ref(value: &MappableKey) -> Option<Self::Value<'_>>;
}

pub struct Borrowed<T>(T);

macro_rules! impl_try_from_for_mappable_key {
    ($variant:ident: $($t:ty),*) => {
        $(
            impl MapKeyBack for $t {
                type Value<'a> = Borrowed<$t>;

                #[inline]
                fn to_ref(value: &MappableKey) -> Option<Self::Value<'_>> {
                    match value {
                        MappableKey::$variant(value) => Some(Borrowed(*value as _)),
                        _ => None,
                    }
                }
            }

            impl IntoBorrowed<$t> for Borrowed<$t> {
                type Borrowed = $t;

                #[inline]
                fn to_borrow(&self) -> &Self::Borrowed {
                    &self.0
                }
            }
        )*
    }
}

impl_from_for_mappable_key!(Str: &str, &String);
impl_from_for_mappable_key!(Int: &i8, &i16, &i32, &i64);
impl_from_for_mappable_key!(UInt: &u8, &u16, &u32, &u64);
impl_from_for_mappable_key!(Bool: &bool);

impl_try_from_for_mappable_key!(Int: i8, i16, i32, i64);
impl_try_from_for_mappable_key!(UInt: u8, u16, u32, u64);
impl_try_from_for_mappable_key!(Bool: bool);

impl IntoBorrowed<String> for Borrowed<&str> {
    type Borrowed = str;

    #[inline]
    fn to_borrow(&self) -> &Self::Borrowed {
        self.0
    }
}
impl MapKeyBack for String {
    type Value<'a> = Borrowed<&'a str>;

    #[inline]
    fn to_ref(value: &MappableKey) -> Option<Self::Value<'_>> {
        match value {
            MappableKey::Str(value) => Some(Borrowed(value)),
            _ => None,
        }
    }
}

impl MappableKey {
    #[allow(private_bounds)]
    pub fn get_value<'a, K, V>(&'a self, map: &'a impl Map<K, V>) -> Option<&'a V>
    where
        K: MapKeyBack,
        K::Value<'a>: IntoBorrowed<K>,
        K: Borrow<<K::Value<'a> as IntoBorrowed<K>>::Borrowed>,
        <K::Value<'a> as IntoBorrowed<K>>::Borrowed: Hash + Eq + Ord,
    {
        let key = K::to_ref(self);
        if let Some(key) = key {
            map.get_ref(key.to_borrow())
        } else {
            None
        }
    }
}
