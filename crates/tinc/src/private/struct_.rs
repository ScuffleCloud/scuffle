use super::{
    DeserializeContent, DeserializeHelper, Expected, IdentifiedValue, Identifier, IdentifierDeserializer, IdentifierFor,
    MapAccessValueDeserializer, SerdePathToken, TrackedError, Tracker, TrackerDeserializeIdentifier, TrackerDeserializer,
    TrackerFor, TrackerValidation, TrackerWrapper, ValidationError, report_tracked_error, set_irrecoverable,
};

pub trait TrackedStructDeserializer<'de>: Sized + TrackerFor + IdentifierFor + Expected
where
    Self::Tracker: TrackerWrapper,
{
    const DENY_UNKNOWN_FIELDS: bool = false;

    fn deserialize<D>(
        &mut self,
        field: Self::Identifier,
        tracker: &mut <Self::Tracker as TrackerWrapper>::Tracker,
        deserializer: D,
    ) -> Result<(), D::Error>
    where
        D: DeserializeContent<'de>;

    fn validate(&self, tracker: &mut <Self::Tracker as TrackerWrapper>::Tracker) -> Result<(), ValidationError>;
}

impl<'de, T> TrackedStructDeserializer<'de> for Box<T>
where
    T: TrackedStructDeserializer<'de> + Default,
    T::Tracker: Tracker<Target = T> + Default + TrackerWrapper,
{
    const DENY_UNKNOWN_FIELDS: bool = T::DENY_UNKNOWN_FIELDS;

    #[inline(always)]
    fn deserialize<D>(
        &mut self,
        field: Self::Identifier,
        tracker: &mut <Self::Tracker as TrackerWrapper>::Tracker,
        deserializer: D,
    ) -> Result<(), D::Error>
    where
        D: DeserializeContent<'de>,
    {
        T::deserialize(self.as_mut(), field, tracker, deserializer)
    }

    #[inline(always)]
    fn validate(&self, tracker: &mut <Self::Tracker as TrackerWrapper>::Tracker) -> Result<(), ValidationError> {
        T::validate(self, tracker)
    }
}

#[derive(Debug, Default)]
pub struct StructTracker<T>(pub T);

impl<T: Tracker> TrackerWrapper for StructTracker<T> {
    type Tracker = T;
}

impl<T> std::ops::Deref for StructTracker<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::DerefMut for StructTracker<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> Tracker for StructTracker<T>
where
    T: Tracker,
{
    type Target = T::Target;

    fn allow_duplicates(&self) -> bool {
        self.0.allow_duplicates()
    }
}

impl<'de, T, S> serde::de::DeserializeSeed<'de> for DeserializeHelper<'_, StructTracker<T>>
where
    T: Tracker<Target = S>,
    S: TrackedStructDeserializer<'de, Tracker = StructTracker<T>>,
{
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_struct(S::NAME, <S::Identifier as Identifier>::OPTIONS, self)
    }
}

impl<T, S> TrackerValidation for StructTracker<T>
where
    T: Tracker<Target = S>,
    S: for<'de> TrackedStructDeserializer<'de, Tracker = StructTracker<T>>,
{
    fn validate(&mut self, value: &Self::Target) -> Result<(), ValidationError> {
        S::validate(value, &mut self.0)
    }
}

impl<'de, T, S> TrackerDeserializer<'de> for StructTracker<T>
where
    T: Tracker<Target = S>,
    S: TrackedStructDeserializer<'de, Tracker = StructTracker<T>>,
{
    fn deserialize<D>(&mut self, value: &mut Self::Target, deserializer: D) -> Result<(), D::Error>
    where
        D: DeserializeContent<'de>,
    {
        deserializer.deserialize_seed(DeserializeHelper { tracker: self, value })
    }
}

impl<'de, T, S> serde::de::Visitor<'de> for DeserializeHelper<'_, StructTracker<T>>
where
    T: Tracker<Target = S>,
    S: TrackedStructDeserializer<'de, Tracker = StructTracker<T>>,
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
            .next_key_seed(IdentifierDeserializer::<S::Identifier>::default())
            .inspect_err(|_| {
                set_irrecoverable();
            })?
        {
            let mut deserialized = false;
            match key {
                IdentifiedValue::Found(field) => {
                    let _token = SerdePathToken::push_field(field.name());
                    S::deserialize(
                        self.value,
                        field,
                        self.tracker,
                        MapAccessValueDeserializer {
                            map: &mut map,
                            deserialized: &mut deserialized,
                        },
                    )?;
                }
                IdentifiedValue::Unknown(field) => {
                    let _token = SerdePathToken::push_field(&field);
                    report_tracked_error(TrackedError::unknown_field(S::DENY_UNKNOWN_FIELDS))?;
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

impl<'de, T> TrackerDeserializeIdentifier<'de> for StructTracker<T>
where
    T: Tracker,
    T::Target: IdentifierFor + TrackedStructDeserializer<'de, Tracker = Self>,
{
    fn deserialize<D>(
        &mut self,
        value: &mut Self::Target,
        identifier: <Self::Target as IdentifierFor>::Identifier,
        deserializer: D,
    ) -> Result<(), D::Error>
    where
        D: DeserializeContent<'de>,
    {
        T::Target::deserialize(value, identifier, self, deserializer)
    }
}
