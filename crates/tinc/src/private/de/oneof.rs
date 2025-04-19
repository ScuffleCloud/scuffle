use std::marker::PhantomData;

use serde::de::{Unexpected, VariantAccess};

use super::buffer::Content;
use super::{
    DeserializeContent, DeserializeHelper, Expected, IdentifiedValue, Identifier, IdentifierDeserializer, IdentifierFor,
    MapAccessValueDeserializer, PathToken, SerdeDeserializer, TrackedError, Tracker, TrackerDeserializer, TrackerFor,
    TrackerValidation, TrackerWrapper, report_error, report_serde_error, set_irrecoverable,
};

pub trait OneOfHelper {
    type Target;
}

impl<T> OneOfHelper for Option<T> {
    type Target = T;
}

pub trait TaggedOneOfIdentifier: Identifier {
    const TAG: Self;
    const CONTENT: Self;
}

pub trait TrackerDeserializeIdentifier<'de>: Tracker
where
    Self::Target: IdentifierFor,
{
    fn deserialize<D>(
        &mut self,
        value: &mut Self::Target,
        identifier: <Self::Target as IdentifierFor>::Identifier,
        deserializer: D,
    ) -> Result<(), D::Error>
    where
        D: DeserializeContent<'de>;
}

pub trait TrackedOneOfVariant {
    type Variant: Identifier;
}

pub trait TrackedOneOfDeserializer<'de>: TrackerFor + IdentifierFor + TrackedOneOfVariant + Sized
where
    Self::Tracker: TrackerWrapper,
{
    const DENY_UNKNOWN_FIELDS: bool = false;

    fn deserialize<D>(
        value: &mut Option<Self>,
        identifier: Self::Variant,
        tracker: &mut Option<<Self::Tracker as TrackerWrapper>::Tracker>,
        deserializer: D,
    ) -> Result<(), D::Error>
    where
        D: DeserializeContent<'de>;

    fn validate<E>(&self, tracker: &mut <Self::Tracker as TrackerWrapper>::Tracker) -> Result<(), E>
    where
        E: serde::de::Error;
}

impl<'de, T> serde::de::Visitor<'de> for DeserializeHelper<'_, TaggedOneOfTracker<T>>
where
    T: Tracker,
    T::Target: TrackedOneOfDeserializer<'de, Tracker = TaggedOneOfTracker<T>>,
    T::Target: IdentifierFor,
    <T::Target as IdentifierFor>::Identifier: TaggedOneOfIdentifier,
{
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        <T::Target as Expected>::expecting(formatter)
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        while let Some(key) = map
            .next_key_seed(IdentifierDeserializer::<<T::Target as IdentifierFor>::Identifier>::new())
            .inspect_err(|_| {
                set_irrecoverable();
            })?
        {
            let _token = PathToken::push_field(match &key {
                IdentifiedValue::Found(tag) => tag.name(),
                IdentifiedValue::Unknown(v) => v.as_ref(),
            });

            let mut deserialized = false;

            match &key {
                IdentifiedValue::Found(tag) => {
                    TrackerDeserializeIdentifier::deserialize(
                        self.tracker,
                        self.value,
                        *tag,
                        MapAccessValueDeserializer {
                            map: &mut map,
                            deserialized: &mut deserialized,
                        },
                    )?;
                }
                IdentifiedValue::Unknown(_) => {
                    report_error(TrackedError::unknown_field(T::Target::DENY_UNKNOWN_FIELDS))?;
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

impl<T> TrackerValidation for OneOfTracker<T>
where
    T: Tracker,
    T::Target: for<'de> TrackedOneOfDeserializer<'de, Tracker = OneOfTracker<T>>,
{
    fn validate<E>(&mut self, value: &Self::Target) -> Result<(), E>
    where
        E: serde::de::Error,
    {
        match (self.0.as_mut(), value) {
            (Some(tracker), Some(value)) => value.validate(tracker),
            (None, Some(_)) => Err(serde::de::Error::custom("tracker not initialized but value is present")),
            (Some(_), None) => Err(serde::de::Error::custom("tracker is present but value is not")),
            (None, None) => Ok(()),
        }
    }
}

impl<'de, T> TrackerDeserializer<'de> for OneOfTracker<T>
where
    T: Tracker,
    T::Target: TrackedOneOfDeserializer<'de, Tracker = OneOfTracker<T>, Variant = <T::Target as IdentifierFor>::Identifier>,
{
    fn deserialize<D>(&mut self, value: &mut Self::Target, deserializer: D) -> Result<(), D::Error>
    where
        D: DeserializeContent<'de>,
    {
        deserializer.deserialize_seed(DeserializeHelper { value, tracker: self })
    }
}

pub struct TrackerForOneOf<T>(PhantomData<T>);

impl<T: TrackerFor> TrackerFor for TrackerForOneOf<T> {
    type Tracker = OneOfTracker<T::Tracker>;
}

const TAGGED_ONE_OF_TRACKER_STATE_TAG_INVALID: u8 = 0b00000001;
const TAGGED_ONE_OF_TRACKER_STATE_HAS_CONTENT: u8 = 0b00000010;

pub struct TaggedOneOfTracker<T>
where
    T: Tracker,
    T::Target: TrackedOneOfVariant,
{
    tracker: Option<T>,
    state: u8,
    tag_buffer: Option<<T::Target as TrackedOneOfVariant>::Variant>,
    content_buffer: Vec<Content<'static>>,
}

impl<T: Tracker> TrackerWrapper for TaggedOneOfTracker<T>
where
    T::Target: TrackedOneOfVariant,
{
    type Tracker = T;
}

impl<'de, T> TrackerDeserializeIdentifier<'de> for TaggedOneOfTracker<T>
where
    T: Tracker,
    T::Target: TrackedOneOfVariant + IdentifierFor,
    <T::Target as IdentifierFor>::Identifier: TaggedOneOfIdentifier,
    T::Target: TrackedOneOfDeserializer<'de, Tracker = TaggedOneOfTracker<T>>,
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
        if identifier == <T::Target as IdentifierFor>::Identifier::TAG {
            let tag = deserializer.deserialize_seed(IdentifierDeserializer::new())?;
            match (tag, self.tag_buffer) {
                (IdentifiedValue::Found(tag), _) if !self.tag_invalid() => {
                    if let Some(existing_tag) = self.tag_buffer {
                        if existing_tag != tag {
                            let error = <D::Error as serde::de::Error>::invalid_value(
                                Unexpected::Str(tag.name()),
                                &existing_tag.name(),
                            );
                            report_serde_error(error)?;
                        }
                    } else {
                        self.tag_buffer = Some(tag);
                    }

                    let _token = PathToken::replace_field(<T::Target as IdentifierFor>::Identifier::CONTENT.name());
                    for content in self.content_buffer.drain(..) {
                        let result: Result<(), D::Error> = T::Target::deserialize(
                            value,
                            tag,
                            &mut self.tracker,
                            SerdeDeserializer {
                                deserializer: serde::de::IntoDeserializer::into_deserializer(content),
                            },
                        );

                        if let Err(e) = result {
                            report_serde_error(e)?;
                        }
                    }
                }
                (IdentifiedValue::Unknown(v), None) => {
                    self.set_tag_invalid();
                    let error = <D::Error as serde::de::Error>::unknown_variant(
                        v.as_ref(),
                        <T::Target as TrackedOneOfVariant>::Variant::OPTIONS,
                    );
                    report_serde_error(error)?;
                }
                (IdentifiedValue::Unknown(v), Some(tag)) => {
                    self.set_tag_invalid();
                    let error = <D::Error as serde::de::Error>::invalid_value(Unexpected::Str(v.as_ref()), &tag.name());
                    report_serde_error(error)?;
                }
                _ => {}
            }
        } else if identifier == <T::Target as IdentifierFor>::Identifier::CONTENT {
            self.set_has_content();
            if !self.tag_invalid() {
                if let Some(tag) = self.tag_buffer {
                    let result: Result<(), D::Error> = T::Target::deserialize(value, tag, &mut self.tracker, deserializer);
                    if let Err(e) = result {
                        report_serde_error(e)?;
                    }
                } else {
                    self.content_buffer.push(
                        deserializer
                            .deserialize::<Content>()
                            .inspect_err(|_| {
                                set_irrecoverable();
                            })?
                            .into_static(),
                    );
                }
            }
        } else {
            report_error(TrackedError::unknown_field(T::Target::DENY_UNKNOWN_FIELDS))?;
        }

        Ok(())
    }
}

impl<T> TrackerValidation for TaggedOneOfTracker<T>
where
    T: Tracker,
    T::Target: for<'de> TrackedOneOfDeserializer<'de, Tracker = TaggedOneOfTracker<T>>,
    <T::Target as IdentifierFor>::Identifier: TaggedOneOfIdentifier,
{
    fn validate<E>(&mut self, value: &Self::Target) -> Result<(), E>
    where
        E: serde::de::Error,
    {
        if !self.content_buffer.is_empty() {
            let _guard = PathToken::push_field(<T::Target as IdentifierFor>::Identifier::TAG.name());
            report_error(TrackedError::missing_field())?;
            return Ok(());
        }

        match (self.tracker.as_mut(), value) {
            (Some(tracker), Some(value)) => {
                let _guard = PathToken::push_field(<T::Target as IdentifierFor>::Identifier::CONTENT.name());
                value.validate(tracker)
            }
            (None, Some(_)) => Err(serde::de::Error::custom("tracker not initialized but value is present")),
            (Some(_), None) => Err(serde::de::Error::custom("tracker is present but value is not")),
            (None, None) => Ok(()),
        }
    }
}

impl<'de, T> TrackerDeserializer<'de> for TaggedOneOfTracker<T>
where
    T: Tracker,
    T::Target: TrackedOneOfDeserializer<'de, Tracker = TaggedOneOfTracker<T>>,
    <T::Target as IdentifierFor>::Identifier: TaggedOneOfIdentifier,
{
    fn deserialize<D>(&mut self, value: &mut Self::Target, deserializer: D) -> Result<(), D::Error>
    where
        D: DeserializeContent<'de>,
    {
        deserializer.deserialize_seed(DeserializeHelper { value, tracker: self })
    }
}

impl<T> std::ops::Deref for TaggedOneOfTracker<T>
where
    T: Tracker,
    T::Target: TrackedOneOfVariant,
{
    type Target = Option<T>;

    fn deref(&self) -> &Self::Target {
        &self.tracker
    }
}

impl<T> std::ops::DerefMut for TaggedOneOfTracker<T>
where
    T: Tracker,
    T::Target: TrackedOneOfVariant,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.tracker
    }
}

impl<T> std::fmt::Debug for TaggedOneOfTracker<T>
where
    T: Tracker + std::fmt::Debug,
    T::Target: TrackedOneOfVariant,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("TaggedOneOfTracker")
            .field("tracker", &self.tracker)
            .field("state", &self.state)
            .field("tag_buffer", &self.tag_buffer.map(|t| t.name()))
            .field("value_buffer", &self.content_buffer)
            .finish()
    }
}

impl<T> Default for TaggedOneOfTracker<T>
where
    T: Tracker,
    T::Target: TrackedOneOfVariant,
{
    fn default() -> Self {
        Self {
            tracker: None,
            state: 0,
            tag_buffer: None,
            content_buffer: Vec::new(),
        }
    }
}

impl<T> TaggedOneOfTracker<T>
where
    T: Tracker,
    T::Target: TrackedOneOfVariant,
{
    pub fn tag_invalid(&self) -> bool {
        self.state & TAGGED_ONE_OF_TRACKER_STATE_TAG_INVALID != 0
    }

    pub fn set_tag_invalid(&mut self) {
        self.state |= TAGGED_ONE_OF_TRACKER_STATE_TAG_INVALID;
    }

    pub fn has_content(&self) -> bool {
        self.state & TAGGED_ONE_OF_TRACKER_STATE_HAS_CONTENT != 0
    }

    pub fn set_has_content(&mut self) {
        self.state |= TAGGED_ONE_OF_TRACKER_STATE_HAS_CONTENT;
    }
}

impl<T> Tracker for TaggedOneOfTracker<T>
where
    T: Tracker,
    T::Target: TrackedOneOfVariant,
{
    type Target = Option<T::Target>;

    fn allow_duplicates(&self) -> bool {
        self.tracker.as_ref().is_none_or(|t| t.allow_duplicates())
    }
}

impl<'de, T> serde::de::DeserializeSeed<'de> for DeserializeHelper<'_, TaggedOneOfTracker<T>>
where
    T: Tracker,
    T::Target: TrackedOneOfDeserializer<'de, Tracker = TaggedOneOfTracker<T>>,
    <T::Target as IdentifierFor>::Identifier: TaggedOneOfIdentifier,
{
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_struct(T::Target::NAME, <T::Target as IdentifierFor>::Identifier::OPTIONS, self)
    }
}

#[derive(Debug)]
pub struct OneOfTracker<T>(pub Option<T>);

impl<T: Tracker> TrackerWrapper for OneOfTracker<T> {
    type Tracker = T;
}

impl<T> std::ops::Deref for OneOfTracker<T> {
    type Target = Option<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::DerefMut for OneOfTracker<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> Default for OneOfTracker<T> {
    fn default() -> Self {
        Self(None)
    }
}

impl<T: Tracker> Tracker for OneOfTracker<T> {
    type Target = Option<T::Target>;

    fn allow_duplicates(&self) -> bool {
        self.0.as_ref().is_none_or(|value| value.allow_duplicates())
    }
}

impl<'de, T> serde::de::DeserializeSeed<'de> for DeserializeHelper<'_, OneOfTracker<T>>
where
    T: Tracker,
    T::Target: TrackedOneOfDeserializer<'de, Tracker = OneOfTracker<T>, Variant = <T::Target as IdentifierFor>::Identifier>,
{
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_enum(T::Target::NAME, <T::Target as IdentifierFor>::Identifier::OPTIONS, self)
    }
}

impl<'de, T> serde::de::Visitor<'de> for DeserializeHelper<'_, OneOfTracker<T>>
where
    T: Tracker,
    T::Target: TrackedOneOfDeserializer<'de, Tracker = OneOfTracker<T>, Variant = <T::Target as IdentifierFor>::Identifier>,
{
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "one of")
    }

    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::EnumAccess<'de>,
    {
        let (variant, variant_access) =
            data.variant_seed(IdentifierDeserializer::<<T::Target as IdentifierFor>::Identifier>::new())?;
        match variant {
            IdentifiedValue::Found(variant) => {
                let _token = PathToken::push_field(variant.name());
                TrackerDeserializeIdentifier::deserialize(
                    self.tracker,
                    self.value,
                    variant,
                    VariantAccessDeserializer { de: variant_access },
                )
            }
            IdentifiedValue::Unknown(variant) => {
                let error = <A::Error as serde::de::Error>::unknown_variant(
                    variant.as_ref(),
                    <T::Target as IdentifierFor>::Identifier::OPTIONS,
                );
                report_serde_error(error)?;
                variant_access.newtype_variant::<serde::de::IgnoredAny>().inspect_err(|_| {
                    set_irrecoverable();
                })?;
                Ok(())
            }
        }
    }
}

impl<'de, T> TrackerDeserializeIdentifier<'de> for OneOfTracker<T>
where
    T: Tracker,
    T::Target: TrackedOneOfDeserializer<'de, Tracker = OneOfTracker<T>, Variant = <T::Target as IdentifierFor>::Identifier>,
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
