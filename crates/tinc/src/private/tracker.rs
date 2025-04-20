use super::{DeserializeContent, Expected};

pub trait Tracker {
    type Target: Expected;

    fn allow_duplicates(&self) -> bool;
}

pub trait TrackerFor {
    type Tracker: Tracker;
}

pub trait TrackerWrapper: Tracker {
    type Tracker: Tracker;
}

pub trait TrackerDeserializer<'de>: Tracker + Sized {
    fn deserialize<D>(&mut self, value: &mut Self::Target, deserializer: D) -> Result<(), D::Error>
    where
        D: DeserializeContent<'de>;
}

pub trait TrackerValidation: Tracker {
    fn validate<E>(&mut self, value: &Self::Target) -> Result<(), E>
    where
        E: serde::de::Error;
}

impl<'de, T> TrackerDeserializer<'de> for Box<T>
where
    T: TrackerDeserializer<'de>,
{
    fn deserialize<D>(&mut self, value: &mut Self::Target, deserializer: D) -> Result<(), D::Error>
    where
        D: DeserializeContent<'de>,
    {
        self.as_mut().deserialize(value, deserializer)
    }
}

impl<T> TrackerValidation for Box<T>
where
    T: TrackerValidation,
{
    fn validate<E>(&mut self, value: &Self::Target) -> Result<(), E>
    where
        E: serde::de::Error,
    {
        self.as_mut().validate(value.as_ref())
    }
}

impl<T: Tracker> Tracker for Box<T> {
    type Target = Box<T::Target>;

    fn allow_duplicates(&self) -> bool {
        self.as_ref().allow_duplicates()
    }
}

impl<T: TrackerFor> TrackerFor for Box<T> {
    type Tracker = Box<T::Tracker>;
}

impl<T: TrackerWrapper> TrackerWrapper for Box<T> {
    type Tracker = T::Tracker;
}
