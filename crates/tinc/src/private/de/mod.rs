use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::marker::PhantomData;
use std::str::FromStr;

pub use wrapper::DeserializerWrapper;

pub mod buffer;
pub mod wrapper;

pub struct TrackerStateGuard {
    state: Option<TrackerSharedState>,
    _marker: PhantomData<*const ()>,
}

impl TrackerStateGuard {
    pub fn new(mut state: TrackerSharedState) -> Self {
        STATE.with_borrow_mut(|current| {
            std::mem::swap(current, &mut state);
            TrackerStateGuard {
                state: Some(state),
                _marker: PhantomData,
            }
        })
    }

    pub fn finish(mut self) -> TrackerSharedState {
        let mut old = self.state.take().unwrap();
        STATE.with_borrow_mut(|current| {
            std::mem::swap(current, &mut old);
        });
        old
    }
}

impl Drop for TrackerStateGuard {
    fn drop(&mut self) {
        if let Some(old) = self.state.take() {
            STATE.with_borrow_mut(|state| {
                *state = old;
            });
        }
    }
}

#[derive(Debug)]
pub enum TrackedErrorKind {
    DuplicateField,
    UnknownField,
    MissingField,
    InvalidField { message: Box<str> },
}

#[derive(Debug)]
pub struct TrackedError {
    pub kind: TrackedErrorKind,
    pub fatal: bool,
    pub path: Box<str>,
}

impl std::fmt::Display for TrackedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            TrackedErrorKind::DuplicateField => write!(f, "`{}` was already provided", self.path),
            TrackedErrorKind::UnknownField => write!(f, "unknown field `{}`", self.path),
            TrackedErrorKind::MissingField => write!(f, "missing field `{}`", self.path),
            TrackedErrorKind::InvalidField { message } => write!(f, "`{}`: {}", self.path, message),
        }
    }
}

impl TrackedError {
    fn new(kind: TrackedErrorKind, fatal: bool) -> Self {
        Self {
            kind,
            fatal,
            path: current_path().into_boxed_str(),
        }
    }

    pub fn unknown_field(fatal: bool) -> Self {
        Self::new(TrackedErrorKind::UnknownField, fatal)
    }

    pub fn invalid_field(message: impl Into<Box<str>>) -> Self {
        Self::new(TrackedErrorKind::InvalidField { message: message.into() }, true)
    }

    pub fn duplicate_field() -> Self {
        Self::new(TrackedErrorKind::DuplicateField, true)
    }

    pub fn missing_field() -> Self {
        Self::new(TrackedErrorKind::MissingField, true)
    }
}

#[derive(Debug)]
pub struct TrackerSharedState {
    pub fail_fast: bool,
    pub irrecoverable: bool,
    pub errors: Vec<TrackedError>,
    pub path_allowed: fn(&[PathItem]) -> bool,
}

impl TrackerSharedState {
    pub fn into_guard(self) -> TrackerStateGuard {
        TrackerStateGuard::new(self)
    }
}

impl Default for TrackerSharedState {
    fn default() -> Self {
        Self {
            fail_fast: true,
            irrecoverable: false,
            errors: Vec::new(),
            path_allowed: |_| true,
        }
    }
}

pub struct MapKey(&'static dyn std::fmt::Display);

impl std::fmt::Debug for MapKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MapKey({})", self.0)
    }
}

#[derive(Debug)]
pub enum PathItem {
    Field(&'static str),
    Index(usize),
    Key(MapKey),
}

pub struct PathToken<'a> {
    _marker: PhantomData<&'a ()>,
    _no_send: PhantomData<*const ()>,
}

fn current_path() -> String {
    PATH_BUFFER.with(|buffer| {
        let mut path = String::new();
        for token in buffer.borrow().iter() {
            match token {
                PathItem::Field(field) => {
                    if !path.is_empty() {
                        path.push('.');
                    }
                    path.push_str(field);
                }
                PathItem::Key(key) => {
                    if !path.is_empty() {
                        path.push('.');
                    }
                    path.push_str(&key.0.to_string());
                }
                PathItem::Index(index) => {
                    path.push('[');
                    path.push_str(&index.to_string());
                    path.push(']');
                }
            }
        }

        path
    })
}

pub fn report_error<E>(error: TrackedError) -> Result<(), E>
where
    E: serde::de::Error,
{
    STATE.with_borrow_mut(|settings| {
        if settings.irrecoverable {
            return Err(E::custom(&error));
        }

        let result = if settings.fail_fast && error.fatal {
            Err(E::custom(&error))
        } else {
            Ok(())
        };

        settings.errors.push(error);
        result
    })
}

fn is_path_allowed() -> bool {
    PATH_BUFFER.with(|buffer| STATE.with_borrow(|settings| (settings.path_allowed)(&buffer.borrow())))
}

#[track_caller]
fn set_irrecoverable() {
    STATE.with_borrow_mut(|settings| settings.irrecoverable = true);
}

impl<'a> PathToken<'a> {
    pub fn push_field(field: &'a str) -> Self {
        PATH_BUFFER.with(|buffer| {
            buffer.borrow_mut().push(PathItem::Field(
                // SAFETY: `field` has a lifetime of `'a`, field-name hides the field so it cannot be accessed outside of this module.
                // We return a `PathToken` that has a lifetime of `'a` which makes it impossible to access this field after its lifetime ends.
                unsafe { std::mem::transmute::<&'a str, &'static str>(field) },
            ))
        });
        Self {
            _marker: PhantomData,
            _no_send: PhantomData,
        }
    }

    pub fn push_index(index: usize) -> Self {
        PATH_BUFFER.with(|buffer| buffer.borrow_mut().push(PathItem::Index(index)));
        Self {
            _marker: PhantomData,
            _no_send: PhantomData,
        }
    }

    pub fn push_key(key: &'a dyn std::fmt::Display) -> Self {
        PATH_BUFFER.with(|buffer| {
            buffer.borrow_mut().push(PathItem::Key(
                // SAFETY: `key` has a lifetime of `'a`, map-key hides the key so it cannot be accessed outside of this module.
                // We return a `PathToken` that has a lifetime of `'a` which makes it impossible to access this key after its lifetime ends.
                MapKey(unsafe { std::mem::transmute::<&'a dyn std::fmt::Display, &'static dyn std::fmt::Display>(key) }),
            ))
        });
        Self {
            _marker: PhantomData,
            _no_send: PhantomData,
        }
    }
}

impl Drop for PathToken<'_> {
    fn drop(&mut self) {
        PATH_BUFFER.with(|buffer| buffer.borrow_mut().pop());
    }
}

thread_local! {
    static PATH_BUFFER: RefCell<Vec<PathItem>> = const { RefCell::new(Vec::new()) };
    static STATE: RefCell<TrackerSharedState> = const {
        RefCell::new(TrackerSharedState {
            fail_fast: true,
            irrecoverable: false,
            errors: Vec::new(),
            path_allowed: |_| true,
        })
    };
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

pub enum IdentifiedValue<'a, F> {
    Found(F),
    Unknown(Cow<'a, str>),
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

pub trait Identifier: FromStr {
    const OPTIONS: &'static [&'static str];
    fn name(&self) -> &'static str;
}

pub trait IdentifierFor {
    const NAME: &'static str;

    type Identifier: Identifier;
}

impl<T: IdentifierFor> IdentifierFor for Box<T> {
    type Identifier = T::Identifier;

    const NAME: &'static str = T::NAME;
}

pub trait TrackedStructDeserializer<'de>: Sized + TrackerFor + IdentifierFor + Expected {
    const DENY_UNKNOWN_FIELDS: bool = false;

    fn deserialize<D>(
        &mut self,
        field: Self::Identifier,
        tracker: &mut Self::Tracker,
        deserializer: D,
    ) -> Result<(), D::Error>
    where
        D: DeserializeContent<'de>;

    fn verify_deserialize<E>(&self, tracker: &mut Self::Tracker) -> Result<(), E>
    where
        E: serde::de::Error,
    {
        let _ = tracker;
        Ok(())
    }
}

impl<'de, T> TrackedStructDeserializer<'de> for Box<T>
where
    T: TrackedStructDeserializer<'de> + Default,
    T::Tracker: Tracker<Target = T> + Default,
{
    const DENY_UNKNOWN_FIELDS: bool = T::DENY_UNKNOWN_FIELDS;

    #[inline(always)]
    fn deserialize<D>(
        &mut self,
        field: Self::Identifier,
        tracker: &mut Self::Tracker,
        deserializer: D,
    ) -> Result<(), D::Error>
    where
        D: DeserializeContent<'de>,
    {
        T::deserialize(self.as_mut(), field, tracker.as_mut(), deserializer)
    }

    #[inline(always)]
    fn verify_deserialize<E>(&self, tracker: &mut Self::Tracker) -> Result<(), E>
    where
        E: serde::de::Error,
    {
        T::verify_deserialize(self, tracker.as_mut())
    }
}

#[derive(Debug, Default)]
pub struct MessageTracker<T> {
    pub value: T,
}

#[derive(Debug, Default)]
pub struct BoxedStructHelper<S, T>(pub Box<T>)
where
    Box<T>: Tracker<Target = Box<S>>;

impl<T> Tracker for MessageTracker<T>
where
    T: Tracker,
{
    type Target = T::Target;

    fn allow_duplicates(&self) -> bool {
        self.value.allow_duplicates()
    }
}

impl<S, T> Tracker for BoxedStructHelper<S, T>
where
    Box<T>: Tracker<Target = Box<S>>,
    S: Default + Expected,
    T: Default,
{
    type Target = Box<S>;

    fn allow_duplicates(&self) -> bool {
        self.0.allow_duplicates()
    }
}

impl<'de, T, S> serde::de::DeserializeSeed<'de> for DeserializeHelper<'_, MessageTracker<T>>
where
    T: Tracker<Target = S>,
    S: TrackedStructDeserializer<'de, Tracker = MessageTracker<T>>,
{
    type Value = ();

    fn deserialize<D>(mut self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_struct(S::NAME, S::Identifier::OPTIONS, &mut self)
    }
}

impl<'de, T, S> serde::de::Visitor<'de> for &mut DeserializeHelper<'_, MessageTracker<T>>
where
    T: Tracker<Target = S>,
    S: TrackedStructDeserializer<'de, Tracker = MessageTracker<T>>,
{
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        S::expecting(formatter)
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        while let Some(key) = map
            .next_key_seed(IdentifierDeserializer(PhantomData::<S::Identifier>))
            .inspect_err(|_| {
                set_irrecoverable();
            })?
        {
            let mut deserialized = false;
            match key {
                IdentifiedValue::Found(field) => {
                    let mut _token = PathToken::push_field(field.name());
                    let result = if is_path_allowed() {
                        S::deserialize(
                            self.value,
                            field,
                            self.tracker,
                            MapAccessValueDeserializer {
                                map: &mut map,
                                deserialized: &mut deserialized,
                            },
                        )
                        .map_err(|e| TrackedError::invalid_field(e.to_string()))
                    } else {
                        Err(TrackedError::unknown_field(S::DENY_UNKNOWN_FIELDS))
                    };

                    if let Err(e) = result {
                        report_error(e)?;
                    }
                }
                IdentifiedValue::Unknown(field) => {
                    let mut _token = PathToken::push_field(&field);
                    report_error(TrackedError::unknown_field(S::DENY_UNKNOWN_FIELDS))?;
                }
            }

            if !deserialized {
                map.next_value::<serde::de::IgnoredAny>().inspect_err(|_| {
                    set_irrecoverable();
                })?;
            }
        }

        Ok(())
    }
}

struct MapAccessValueDeserializer<'a, T> {
    map: &'a mut T,
    deserialized: &'a mut bool,
}

impl<'de, M> DeserializeContent<'de> for MapAccessValueDeserializer<'_, M>
where
    M: serde::de::MapAccess<'de>,
{
    type Error = M::Error;

    fn deserialize_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        if *self.deserialized {
            return Err(serde::de::Error::custom("invalid state: field already deserialized"));
        }

        *self.deserialized = true;
        self.map.next_value_seed(seed)
    }
}

pub trait DeserializeContent<'de>: Sized {
    type Error: serde::de::Error;

    fn deserialize<T>(self) -> Result<T, Self::Error>
    where
        T: serde::de::Deserialize<'de>,
    {
        self.deserialize_seed(PhantomData)
    }

    fn deserialize_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>;
}

struct SerdeDeserializer<D> {
    deserializer: D,
}

impl<'de, D> DeserializeContent<'de> for SerdeDeserializer<D>
where
    D: serde::Deserializer<'de>,
{
    type Error = D::Error;

    fn deserialize_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        seed.deserialize(self.deserializer)
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! __tinc_field_from_str {
    (
        $s:expr,
        $($literal:literal => $expr:expr),*$(,)?
    ) => {
        match $s {
            $($literal => Ok($expr),)*
            _ => Err(()),
        }
    };
}

pub use tinc_derive::TincMessageTracker;

use super::well_known::WellKnownAlias;

pub trait Tracker {
    type Target: Expected;

    fn allow_duplicates(&self) -> bool;
}

impl<T: Tracker> Tracker for Box<T> {
    type Target = Box<T::Target>;

    fn allow_duplicates(&self) -> bool {
        self.as_ref().allow_duplicates()
    }
}

pub trait TrackerFor {
    type Tracker: Tracker;
}

impl<T: TrackerFor> TrackerFor for Box<T> {
    type Tracker = Box<T::Tracker>;
}

pub struct PrimitiveTracker<T>(PhantomData<T>);

impl<T> std::fmt::Debug for PrimitiveTracker<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PrimitiveTracker<{}>", std::any::type_name::<T>())
    }
}

impl<T> Default for PrimitiveTracker<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

pub trait Expected {
    fn expecting(formatter: &mut std::fmt::Formatter) -> std::fmt::Result;
}

impl<V: Expected> Expected for Box<V> {
    fn expecting(formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        V::expecting(formatter)
    }
}

impl<V: Expected> Expected for Option<V> {
    fn expecting(formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "an optional `")?;
        V::expecting(formatter)?;
        write!(formatter, "`")
    }
}

impl<V: Expected> Expected for Vec<V> {
    fn expecting(formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a sequence of `")?;
        V::expecting(formatter)?;
        write!(formatter, "`s")
    }
}

impl<K: Expected, V: Expected> Expected for BTreeMap<K, V> {
    fn expecting(formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a map of `")?;
        K::expecting(formatter)?;
        write!(formatter, "`s to `")?;
        V::expecting(formatter)?;
        write!(formatter, "`s")
    }
}

impl<K: Expected, V: Expected, S> Expected for HashMap<K, V, S> {
    fn expecting(formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a map of `")?;
        K::expecting(formatter)?;
        write!(formatter, "`s to `")?;
        V::expecting(formatter)?;
        write!(formatter, "`s")
    }
}

impl<T: Default + Expected> Tracker for PrimitiveTracker<T> {
    type Target = T;

    #[inline(always)]
    fn allow_duplicates(&self) -> bool {
        false
    }
}

macro_rules! impl_tracker_for_primitive {
    ($($ty:ty),*) => {
        $(
            impl TrackerFor for $ty {
                type Tracker = PrimitiveTracker<$ty>;
            }

            impl Expected for $ty {
                fn expecting(formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    write!(formatter, stringify!($ty))
                }
            }
        )*
    };
}

impl_tracker_for_primitive!(String, bool, u8, u16, u32, u64, i8, i16, i32, i64, f32, f64);

#[derive(Debug)]
pub struct OptionalTracker<T>(pub Option<T>);

impl<T> Default for OptionalTracker<T> {
    fn default() -> Self {
        Self(None)
    }
}

impl<T: Tracker> Tracker for OptionalTracker<T> {
    type Target = Option<T::Target>;

    fn allow_duplicates(&self) -> bool {
        self.0.as_ref().map(|t| t.allow_duplicates()).unwrap_or(false)
    }
}

impl<T: TrackerFor> TrackerFor for Option<T> {
    type Tracker = OptionalTracker<T::Tracker>;
}

#[derive(Debug)]
pub struct RepeatedVecTracker<T> {
    pub vec: Vec<T>,
}

impl<T> Default for RepeatedVecTracker<T> {
    fn default() -> Self {
        Self { vec: Default::default() }
    }
}

impl<T: Tracker> Tracker for RepeatedVecTracker<T> {
    type Target = Vec<T::Target>;

    #[inline(always)]
    fn allow_duplicates(&self) -> bool {
        false
    }
}

impl<T: TrackerFor> TrackerFor for Vec<T> {
    type Tracker = RepeatedVecTracker<T::Tracker>;
}

pub struct MapTracker<K: Eq, V, M> {
    pub map: linear_map::LinearMap<K, V>,
    _marker: PhantomData<M>,
}

impl<K: Eq + std::fmt::Debug, V: std::fmt::Debug, M> std::fmt::Debug for MapTracker<K, V, M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut map = f.debug_map();
        for (key, value) in &self.map {
            map.entry(key, value);
        }
        map.finish()
    }
}

impl<K: Eq, V, M> Default for MapTracker<K, V, M> {
    fn default() -> Self {
        Self {
            map: Default::default(),
            _marker: PhantomData,
        }
    }
}

pub trait Map<K, V> {
    fn get_mut(&mut self, key: &K) -> Option<&mut V>;
    fn insert(&mut self, key: K, value: V);
    fn reserve(&mut self, additional: usize);
}

impl<K: Eq, V: Tracker, M: Default + Expected> Tracker for MapTracker<K, V, M> {
    type Target = M;

    fn allow_duplicates(&self) -> bool {
        true
    }
}

impl<K: std::hash::Hash + Eq + Expected, V: TrackerFor + Default + Expected, S: Default> TrackerFor for HashMap<K, V, S> {
    type Tracker = MapTracker<K, V::Tracker, HashMap<K, <V::Tracker as Tracker>::Target, S>>;
}

impl<K: Ord + Expected, V: TrackerFor + Default + Expected> TrackerFor for BTreeMap<K, V> {
    type Tracker = MapTracker<K, V::Tracker, BTreeMap<K, <V::Tracker as Tracker>::Target>>;
}

impl<K: std::hash::Hash + Eq, V: Default> Map<K, V> for HashMap<K, V> {
    fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.get_mut(key)
    }

    fn insert(&mut self, key: K, value: V) {
        self.insert(key, value);
    }

    fn reserve(&mut self, additional: usize) {
        self.reserve(additional);
    }
}

impl<K: Ord, V: Default> Map<K, V> for BTreeMap<K, V> {
    fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.get_mut(key)
    }

    fn insert(&mut self, key: K, value: V) {
        self.insert(key, value);
    }

    fn reserve(&mut self, _: usize) {}
}

pub struct DeserializeHelper<'a, T: Tracker> {
    pub value: &'a mut T::Target,
    pub tracker: &'a mut T,
}

impl<'de, T: Tracker> serde::de::DeserializeSeed<'de> for DeserializeHelper<'_, Box<T>>
where
    for<'a> DeserializeHelper<'a, T>: serde::de::DeserializeSeed<'de, Value = ()>,
{
    type Value = ();

    fn deserialize<D>(self, de: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        DeserializeHelper {
            value: self.value.as_mut(),
            tracker: self.tracker.as_mut(),
        }
        .deserialize(de)
    }
}

impl<'de, T> serde::de::DeserializeSeed<'de> for DeserializeHelper<'_, PrimitiveTracker<T>>
where
    T: serde::Deserialize<'de>,
    PrimitiveTracker<T>: Tracker<Target = T>,
{
    type Value = ();

    fn deserialize<D>(self, de: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        *self.value = serde::Deserialize::deserialize(de)?;
        Ok(())
    }
}

impl<'de, T> serde::de::DeserializeSeed<'de> for DeserializeHelper<'_, RepeatedVecTracker<T>>
where
    for<'a> DeserializeHelper<'a, T>: serde::de::DeserializeSeed<'de, Value = ()>,
    T: Tracker + Default,
    T::Target: Default,
{
    type Value = ();

    fn deserialize<D>(self, de: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        de.deserialize_seq(self)
    }
}

impl<'de, T> serde::de::Visitor<'de> for DeserializeHelper<'_, RepeatedVecTracker<T>>
where
    for<'a> DeserializeHelper<'a, T>: serde::de::DeserializeSeed<'de, Value = ()>,
    T: Tracker + Default,
    T::Target: Default,
{
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        Vec::<T::Target>::expecting(formatter)
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut value = T::Target::default();
        let mut tracker = T::default();

        if let Some(size) = seq.size_hint() {
            self.tracker.vec.reserve(size);
            self.value.reserve(size);
        }

        let mut index = 0;

        loop {
            let mut _token = PathToken::push_index(index);

            let Some(result) = seq
                .next_element_seed(DeserializeHelper {
                    value: &mut value,
                    tracker: &mut tracker,
                })
                .transpose()
            else {
                break;
            };

            if let Err(error) = result {
                report_error(TrackedError::invalid_field(error.to_string()))?;
            }

            self.value.push(std::mem::take(&mut value));
            self.tracker.vec.push(std::mem::take(&mut tracker));
            index += 1;
        }

        Ok(())
    }
}

impl<'de, K, T, M> serde::de::DeserializeSeed<'de> for DeserializeHelper<'_, MapTracker<K, T, M>>
where
    for<'a> DeserializeHelper<'a, T>: serde::de::DeserializeSeed<'de, Value = ()>,
    T: Tracker + Default,
    K: serde::de::Deserialize<'de> + std::cmp::Eq + Clone + std::fmt::Display + Expected,
    M: Map<K, T::Target>,
    MapTracker<K, T, M>: Tracker<Target = M>,
    T::Target: Default,
{
    type Value = ();

    fn deserialize<D>(self, de: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        de.deserialize_map(self)
    }
}

impl<'de, K, T, M> serde::de::Visitor<'de> for DeserializeHelper<'_, MapTracker<K, T, M>>
where
    for<'a> DeserializeHelper<'a, T>: serde::de::DeserializeSeed<'de, Value = ()>,
    T: Tracker + Default,
    K: serde::de::Deserialize<'de> + std::cmp::Eq + Clone + std::fmt::Display + Expected,
    M: Map<K, T::Target>,
    MapTracker<K, T, M>: Tracker<Target = M>,
    T::Target: Default,
{
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        HashMap::<K, T::Target>::expecting(formatter)
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        if let Some(size) = map.size_hint() {
            self.tracker.map.reserve(size);
            self.value.reserve(size);
        }

        let mut new_value = T::Target::default();

        while let Some(key) = map.next_key::<K>().inspect_err(|_| {
            set_irrecoverable();
        })? {
            let mut _token = PathToken::push_key(&key);
            let entry = self.tracker.map.entry(key.clone());
            if let linear_map::Entry::Occupied(entry) = &entry {
                if !entry.get().allow_duplicates() {
                    report_error(TrackedError::duplicate_field())?;
                    map.next_value::<serde::de::IgnoredAny>().inspect_err(|_| {
                        set_irrecoverable();
                    })?;
                    continue;
                }
            }

            let tracker = entry.or_insert_with(Default::default);
            let value = self.value.get_mut(&key);
            let used_new = value.is_none();
            let value = value.unwrap_or(&mut new_value);
            match map.next_value_seed(DeserializeHelper { value, tracker }) {
                Ok(_) => {}
                Err(error) => {
                    report_error(TrackedError::invalid_field(error.to_string()))?;
                    break;
                }
            }

            drop(_token);

            if used_new {
                self.value.insert(key, std::mem::take(&mut new_value));
            }
        }

        Ok(())
    }
}

impl<'de, T> serde::de::DeserializeSeed<'de> for DeserializeHelper<'_, OptionalTracker<T>>
where
    for<'a> DeserializeHelper<'a, T>: serde::de::DeserializeSeed<'de, Value = ()>,
    T: Tracker + Default,
    T::Target: Default,
{
    type Value = ();

    fn deserialize<D>(self, de: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if let Some(value) = self.value {
            DeserializeHelper {
                value,
                tracker: self.tracker.0.get_or_insert_default(),
            }
            .deserialize(de)
        } else {
            de.deserialize_option(self)
        }
    }
}

impl<'de, T> serde::de::Visitor<'de> for DeserializeHelper<'_, OptionalTracker<T>>
where
    for<'a> DeserializeHelper<'a, T>: serde::de::DeserializeSeed<'de, Value = ()>,
    T: Tracker + Default,
    T::Target: Default,
{
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        Option::<T::Target>::expecting(formatter)
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(())
    }

    fn visit_some<D>(self, de: D) -> Result<Self::Value, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        serde::de::DeserializeSeed::deserialize(
            DeserializeHelper {
                value: self.value.get_or_insert_default(),
                tracker: self.tracker.0.get_or_insert_default(),
            },
            de,
        )
    }
}

pub struct EnumTracker<T>(PhantomData<T>);

impl<T> std::fmt::Debug for EnumTracker<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EnumTracker<{}>", std::any::type_name::<T>())
    }
}

impl<T> Default for EnumTracker<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<T> Tracker for EnumTracker<T> {
    type Target = i32;

    fn allow_duplicates(&self) -> bool {
        false
    }
}

pub struct Enum<T> {
    value: i32,
    _marker: PhantomData<T>,
}

impl<T: TryFrom<i32> + Default + std::fmt::Debug> std::fmt::Debug for Enum<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Enum({:?})", T::try_from(self.value).unwrap_or_default())
    }
}

impl<T: Expected> Expected for Enum<T> {
    fn expecting(formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "an enum of `")?;
        T::expecting(formatter)?;
        write!(formatter, "`")
    }
}

impl<T> Default for Enum<T> {
    fn default() -> Self {
        Self {
            value: Default::default(),
            _marker: PhantomData,
        }
    }
}

impl<T> TrackerFor for Enum<T> {
    type Tracker = EnumTracker<T>;
}

pub trait EnumHelper {
    type Target<E>;
}

impl EnumHelper for i32 {
    type Target<E> = Enum<E>;
}

impl EnumHelper for Option<i32> {
    type Target<E> = Option<Enum<E>>;
}

impl EnumHelper for Vec<i32> {
    type Target<E> = Vec<Enum<E>>;
}

impl<K: Ord> EnumHelper for BTreeMap<K, i32> {
    type Target<E> = BTreeMap<K, Enum<E>>;
}

impl<K, S> EnumHelper for HashMap<K, i32, S> {
    type Target<E> = HashMap<K, Enum<E>, S>;
}

impl<'de, T> serde::de::DeserializeSeed<'de> for DeserializeHelper<'_, EnumTracker<T>>
where
    T: serde::Deserialize<'de> + Into<i32>,
{
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        *self.value = T::deserialize(deserializer)?.into();
        Ok(())
    }
}

#[inline(always)]
pub fn tracker_allow_duplicates<T: Tracker>(tracker: Option<&T>) -> bool {
    tracker.is_none_or(|tracker| tracker.allow_duplicates())
}

pub struct WellKnownTracker<T>(PhantomData<T>);

impl<T> std::fmt::Debug for WellKnownTracker<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "WellKnownTracker<{}>", std::any::type_name::<T>())
    }
}

impl<T: Expected> Expected for WellKnownTracker<T> {
    fn expecting(formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        T::expecting(formatter)
    }
}

impl<T> Default for WellKnownTracker<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<T: Default + Expected> Tracker for WellKnownTracker<T> {
    type Target = T;

    fn allow_duplicates(&self) -> bool {
        false
    }
}

impl TrackerFor for prost_types::Struct {
    type Tracker = WellKnownTracker<prost_types::Struct>;
}

impl Expected for prost_types::Struct {
    fn expecting(formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "struct")
    }
}

impl TrackerFor for prost_types::ListValue {
    type Tracker = WellKnownTracker<prost_types::ListValue>;
}

impl Expected for prost_types::ListValue {
    fn expecting(formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "list")
    }
}

impl TrackerFor for prost_types::Timestamp {
    type Tracker = WellKnownTracker<prost_types::Timestamp>;
}

impl Expected for prost_types::Timestamp {
    fn expecting(formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "timestamp")
    }
}

impl TrackerFor for prost_types::Duration {
    type Tracker = WellKnownTracker<prost_types::Duration>;
}

impl Expected for prost_types::Duration {
    fn expecting(formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "duration")
    }
}

impl TrackerFor for prost_types::Value {
    type Tracker = WellKnownTracker<prost_types::Value>;
}

impl Expected for prost_types::Value {
    fn expecting(formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "value")
    }
}

impl TrackerFor for () {
    type Tracker = WellKnownTracker<()>;
}

impl Expected for () {
    fn expecting(formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "empty object")
    }
}

impl<'de, T> serde::de::DeserializeSeed<'de> for DeserializeHelper<'_, WellKnownTracker<T>>
where
    T: WellKnownAlias + Default + Expected,
    T::Helper: serde::Deserialize<'de>,
{
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value: T::Helper = serde::Deserialize::deserialize(deserializer)?;
        *self.value = T::reverse_cast(value);
        Ok(())
    }
}

#[derive(Debug)]
pub struct OneOfTracker<T> {
    pub value: Option<T>,
}

impl<T> Default for OneOfTracker<T> {
    fn default() -> Self {
        Self { value: None }
    }
}

impl<T: Tracker> Tracker for OneOfTracker<T> {
    type Target = Option<T::Target>;

    fn allow_duplicates(&self) -> bool {
        self.value.as_ref().is_none_or(|value| value.allow_duplicates())
    }
}

impl<'de, T> serde::de::DeserializeSeed<'de> for DeserializeHelper<'_, OneOfTracker<T>>
where
    T: Tracker,
    T::Target: TrackedOneOfDeserializer<'de, Tracker = T>,
{
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if let Some(tag_content) = T::Target::OPTIONS.tag_content {
            deserializer.deserialize_struct(
                T::Target::NAME,
                tag_content,
                OneOfTaggedStyle {
                    value: self.value,
                    tracker: &mut self.tracker.value,
                    tag_key: tag_content[0],
                    content_key: tag_content[1],
                },
            )
        } else {
            deserializer.deserialize_enum(
                T::Target::NAME,
                <T::Target as IdentifierFor>::Identifier::OPTIONS,
                OneOfEnumStyle {
                    value: self.value,
                    tracker: &mut self.tracker.value,
                },
            )
        }
    }
}

pub struct OneOfEnumStyle<'a, T: TrackerFor> {
    value: &'a mut Option<T>,
    tracker: &'a mut Option<T::Tracker>,
}

impl<'de, T> serde::de::Visitor<'de> for OneOfEnumStyle<'_, T>
where
    T: TrackedOneOfDeserializer<'de>,
{
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "one of")
    }

    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::EnumAccess<'de>,
    {
        let (variant, variant_access) = data.variant_seed(IdentifierDeserializer::<T::Identifier>::new())?;
        match variant {
            IdentifiedValue::Found(variant) => T::deserialize(
                self.value,
                self.tracker,
                variant,
                VariantAccessDeserializer { de: variant_access },
            ),
            IdentifiedValue::Unknown(variant) => {
                todo!("unknown variant: {}", variant)
            }
        }
    }
}

struct VariantAccessDeserializer<D> {
    de: D,
}

impl<'de, D> DeserializeContent<'de> for VariantAccessDeserializer<D>
where
    D: serde::de::VariantAccess<'de>,
{
    type Error = D::Error;

    fn deserialize_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        self.de.newtype_variant_seed(seed)
    }
}

pub struct OneOfTaggedStyle<'a, T: TrackerFor> {
    value: &'a mut Option<T>,
    tracker: &'a mut Option<T::Tracker>,
    tag_key: &'static str,
    content_key: &'static str,
}

impl<'de, T> serde::de::Visitor<'de> for OneOfTaggedStyle<'_, T>
where
    T: TrackedOneOfDeserializer<'de>,
{
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "one of")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        enum TagOrContent<'de> {
            Tag,
            Content,
            Unknown(Cow<'de, str>),
        }

        struct IdentifierVisitor {
            tag: &'static str,
            content: &'static str,
        }

        impl IdentifierVisitor {
            fn visit_owned_borrowed_or_ref<'de>(self, v: OwnedBorrowedOrRef<'de, '_>) -> TagOrContent<'de> {
                if v.as_ref() == self.tag {
                    TagOrContent::Tag
                } else if v.as_ref() == self.content {
                    TagOrContent::Content
                } else {
                    TagOrContent::Unknown(v.into_cow())
                }
            }
        }

        impl<'de> serde::de::Visitor<'de> for IdentifierVisitor {
            type Value = TagOrContent<'de>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "{} or {}", self.tag, self.content)
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(self.visit_owned_borrowed_or_ref(OwnedBorrowedOrRef::Ref(v)))
            }

            fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(self.visit_owned_borrowed_or_ref(OwnedBorrowedOrRef::Borrowed(v)))
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(self.visit_owned_borrowed_or_ref(OwnedBorrowedOrRef::Owned(v)))
            }
        }

        impl<'de> serde::de::DeserializeSeed<'de> for IdentifierVisitor {
            type Value = TagOrContent<'de>;

            fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                deserializer.deserialize_identifier(self)
            }
        }

        let mut tag_buffer = None;
        let mut content_buffer: Option<buffer::Value<'_>> = None;
        let mut has_content = false;
        let mut has_tag = false;
        while let Some(key) = map
            .next_key_seed(IdentifierVisitor {
                tag: self.tag_key,
                content: self.content_key,
            })
            .inspect_err(|_| {
                set_irrecoverable();
            })?
        {
            let _token = PathToken::push_field(match &key {
                TagOrContent::Tag => self.tag_key,
                TagOrContent::Content => self.content_key,
                TagOrContent::Unknown(v) => v.as_ref(),
            });

            match &key {
                TagOrContent::Tag if !has_tag => {
                    has_tag = true;
                    let tag = map.next_value_seed(IdentifierDeserializer::<T::Identifier>::new())?;
                    match content_buffer.take() {
                        None => {
                            tag_buffer = Some(tag);
                        }
                        Some(content) => match tag {
                            IdentifiedValue::Found(tag) => {
                                drop(_token);
                                let _token = PathToken::push_field(self.content_key);
                                let result: Result<(), A::Error> = T::deserialize(
                                    self.value,
                                    self.tracker,
                                    tag,
                                    SerdeDeserializer {
                                        deserializer: serde::de::IntoDeserializer::into_deserializer(content),
                                    },
                                );

                                if let Err(e) = result {
                                    report_error(TrackedError::invalid_field(e.to_string()))?;
                                }
                            }
                            IdentifiedValue::Unknown(v) => {
                                let error =
                                    <A::Error as serde::de::Error>::unknown_variant(v.as_ref(), T::Identifier::OPTIONS);
                                report_error(TrackedError::invalid_field(error.to_string()))?;
                            }
                        },
                    }
                }
                TagOrContent::Content if !has_content => {
                    has_content = true;
                    match tag_buffer.take() {
                        Some(IdentifiedValue::Found(tag)) => {
                            let mut deserialized = false;
                            T::deserialize(
                                self.value,
                                self.tracker,
                                tag,
                                MapAccessValueDeserializer {
                                    map: &mut map,
                                    deserialized: &mut deserialized,
                                },
                            )?;

                            if !deserialized {
                                map.next_value::<serde::de::IgnoredAny>().inspect_err(|_| {
                                    set_irrecoverable();
                                })?;
                            }
                        }
                        Some(IdentifiedValue::Unknown(v)) => {
                            drop(_token);
                            let _token = PathToken::push_field(self.tag_key);
                            let error = <A::Error as serde::de::Error>::unknown_variant(v.as_ref(), T::Identifier::OPTIONS);
                            report_error(TrackedError::invalid_field(error.to_string()))?;
                            map.next_value::<serde::de::IgnoredAny>().inspect_err(|_| {
                                set_irrecoverable();
                            })?;
                        }
                        None => {
                            content_buffer = Some(map.next_value::<buffer::Value>().inspect_err(|_| {
                                set_irrecoverable();
                            })?);
                        }
                    }
                }
                TagOrContent::Content => {
                    report_error(TrackedError::duplicate_field())?;
                    map.next_value::<serde::de::IgnoredAny>().inspect_err(|_| {
                        set_irrecoverable();
                    })?;
                }
                TagOrContent::Tag => {
                    report_error(TrackedError::duplicate_field())?;
                    map.next_value::<serde::de::IgnoredAny>().inspect_err(|_| {
                        set_irrecoverable();
                    })?;
                }
                TagOrContent::Unknown(_) => {
                    report_error(TrackedError::unknown_field(T::DENY_UNKNOWN_FIELDS))?;
                    map.next_value::<serde::de::IgnoredAny>().inspect_err(|_| {
                        set_irrecoverable();
                    })?;
                }
            }
        }

        Ok(())
    }
}

pub struct OneofOptions {
    /// the tag & content field if the enum is adjacently tagged
    pub tag_content: Option<&'static [&'static str; 2]>,
}

pub trait TrackedOneOfDeserializer<'de>: TrackerFor + IdentifierFor + Sized {
    const OPTIONS: OneofOptions = OneofOptions { tag_content: None };

    const DENY_UNKNOWN_FIELDS: bool = false;

    fn deserialize<D>(
        value: &mut Option<Self>,
        tracker: &mut Option<Self::Tracker>,
        variant: Self::Identifier,
        deserializer: D,
    ) -> Result<(), D::Error>
    where
        D: DeserializeContent<'de>;
}

pub trait OneOfHelper {
    type Target;
}

impl<T> OneOfHelper for Option<T> {
    type Target = TrackerForOneOf<T>;
}

pub struct TrackerForOneOf<T>(PhantomData<T>);

impl<T: TrackerFor> TrackerFor for TrackerForOneOf<T> {
    type Tracker = OneOfTracker<T::Tracker>;
}
