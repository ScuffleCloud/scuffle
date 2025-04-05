use std::sync::Arc;

use map::TrackerMap;
use map_struct::TrackerMapStruct;
use repeated::TrackerRepeated;
use repeated_struct::TrackerRepeatedStruct;

use self::struct_::TrackerStruct;
use super::map::MappableKey;

pub mod map;
pub mod map_struct;
pub mod repeated;
pub mod repeated_struct;
pub mod struct_;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shared {
    pub fail_fast: bool,
    pub failed: bool,
    pub source: Option<Arc<str>>,
}

impl Default for Shared {
    fn default() -> Self {
        Self {
            fail_fast: true,
            failed: false,
            source: None,
        }
    }
}

pub struct Tracker<'a, T> {
    pub inner: &'a mut T,
    pub shared: &'a mut Shared,
}

impl<T> Tracker<'_, T> {
    #[allow(private_bounds)]
    pub fn report_error<E>(&mut self, location: Option<ErrorLocation>, error: E) -> Result<(), E>
    where
        T: StoreError,
        E: serde::de::Error,
    {
        if !self.shared.fail_fast {
            let boxed = error.to_string().into_boxed_str();
            self.inner.store_error(TrackerError { location, source: self.shared.source.clone(), error: boxed });
        }

        self.shared.failed = true;
        if self.shared.fail_fast { Err(error) } else { Ok(()) }
    }
}

impl StoreError for TrackerAny {
    fn store_error(&mut self, error: TrackerError) {
        match self {
            Self::Struct(tracker) => tracker.store_error(error),
            Self::MapStruct(tracker) => tracker.store_error(error),
            Self::Map(tracker) => tracker.store_error(error),
            Self::RepeatedStruct(tracker) => tracker.store_error(error),
            Self::Repeated(tracker) => tracker.store_error(error),
        }
    }
}

#[derive(Debug, Clone)]
pub enum TrackerAny {
    Struct(TrackerStruct),
    MapStruct(TrackerMapStruct),
    Map(TrackerMap),
    RepeatedStruct(TrackerRepeatedStruct),
    Repeated(TrackerRepeated),
}

impl TrackerAny {
    pub fn name(&self) -> &'static str {
        match self {
            TrackerAny::Struct(_) => "Struct",
            TrackerAny::MapStruct(_) => "MapStruct",
            TrackerAny::Map(_) => "Map",
            TrackerAny::RepeatedStruct(_) => "RepeatedStruct",
            TrackerAny::Repeated(_) => "Repeated",
        }
    }
}

#[derive(Debug, Clone)]
pub enum ErrorLocation {
    StructField { name: &'static str },
    MapKey { key: MappableKey },
    SequenceIndex { index: usize },
}

#[derive(Debug, Clone)]
pub struct TrackerError {
    pub location: Option<ErrorLocation>,
    pub source: Option<Arc<str>>,
    pub error: Box<str>,
}

trait StoreError {
    fn store_error(&mut self, error: TrackerError);
}
