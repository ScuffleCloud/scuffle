use super::{ErrorLocation, StoreError, Tracker, TrackerError};

#[derive(Debug, Clone, Default)]
pub struct TrackerRepeated {
    pub errors: Vec<TrackerError>,
}

impl StoreError for TrackerRepeated {
    fn store_error(&mut self, error: TrackerError) {
        self.errors.push(error);
    }
}

pub struct RepeatedDeserializer<'a, V, U> {
    value: &'a mut Vec<V>,
    tracker: Tracker<'a, TrackerRepeated>,
    _marker: std::marker::PhantomData<U>,
}

impl<'a, V> RepeatedDeserializer<'a, V, V> {
    #[inline]
    pub fn new(value: &'a mut Vec<V>, tracker: Tracker<'a, TrackerRepeated>) -> Self {
        Self {
            value,
            tracker,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<'a, U> RepeatedDeserializer<'a, i32, U> {
    #[inline]
    pub fn new_with_helper(value: &'a mut Vec<i32>, tracker: Tracker<'a, TrackerRepeated>, _: std::marker::PhantomData<U>) -> Self {
        Self {
            value,
            tracker,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<'de, V, U> serde::de::DeserializeSeed<'de> for RepeatedDeserializer<'_, V, U>
where
    V: From<U>,
    U: serde::Deserialize<'de>,
{
    type Value = ();

    #[inline]
    fn deserialize<D>(mut self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(&mut self)
    }
}

impl<'de, V, U> serde::de::Visitor<'de> for &mut RepeatedDeserializer<'_, V, U>
where
    V: From<U>,
    U: serde::Deserialize<'de>,
{
    type Value = ();

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        if let Some(size) = seq.size_hint() {
            self.value.reserve(size);
        }

        let mut idx = 0;

        while let Some(element) = seq.next_element::<U>().transpose() {
            match element {
                Ok(element) => self.value.push(V::from(element)),
                Err(err) => {
                    self.tracker
                        .report_error(Some(ErrorLocation::SequenceIndex { index: idx }), err)?;
                    break;
                }
            }
            idx += 1;
        }

        Ok(())
    }

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an array")
    }
}
