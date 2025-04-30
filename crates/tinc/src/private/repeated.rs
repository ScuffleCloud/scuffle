use super::{
    DeserializeHelper, Expected, ProtoPathToken, SerdePathToken, Tracker, TrackerDeserializer, TrackerFor,
    TrackerValidation, ValidationError, report_de_error,
};

#[derive(Debug)]
pub struct RepeatedVecTracker<T>(Vec<T>);

impl<T> std::ops::Deref for RepeatedVecTracker<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::DerefMut for RepeatedVecTracker<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> Default for RepeatedVecTracker<T> {
    fn default() -> Self {
        Self(Default::default())
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
            self.tracker.reserve(size);
            self.value.reserve(size);
        }

        let mut index = 0;

        loop {
            let _token = (SerdePathToken::push_index(index), ProtoPathToken::push_index(index));

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
                report_de_error(error)?;
                break;
            }

            self.value.push(std::mem::take(&mut value));
            self.tracker.push(std::mem::take(&mut tracker));
            index += 1;
        }

        Ok(())
    }
}

impl<'de, T> TrackerDeserializer<'de> for RepeatedVecTracker<T>
where
    for<'a> DeserializeHelper<'a, T>: serde::de::DeserializeSeed<'de, Value = ()>,
    T: Tracker + Default,
    T::Target: Default,
{
    fn deserialize<D>(&mut self, value: &mut Self::Target, deserializer: D) -> Result<(), D::Error>
    where
        D: super::DeserializeContent<'de>,
    {
        deserializer.deserialize_seed(DeserializeHelper { value, tracker: self })
    }
}

impl<T> TrackerValidation for RepeatedVecTracker<T>
where
    T: Tracker + Default + TrackerValidation,
    T::Target: Default,
{
    fn validate(&mut self, value: &Self::Target) -> Result<(), ValidationError> {
        for (index, (value, tracker)) in value.iter().zip(self.iter_mut()).enumerate() {
            let _token = (SerdePathToken::push_index(index), ProtoPathToken::push_index(index));
            tracker.validate(value)?;
        }

        Ok(())
    }
}
