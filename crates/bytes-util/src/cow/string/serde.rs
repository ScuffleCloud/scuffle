use super::StringCow;

impl serde::Serialize for StringCow<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> serde::Deserialize<'de> for StringCow<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct StringCowVisitor;

        impl<'de> serde::de::Visitor<'de> for StringCowVisitor {
            type Value = StringCow<'de>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string")
            }

            fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(StringCow::from_ref(v))
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(StringCow::from_string(v))
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(StringCow::from_string(v.to_string()))
            }
        }

        deserializer.deserialize_any(StringCowVisitor)
    }
}

impl<'de, E> serde::de::IntoDeserializer<'de, E> for StringCow<'de>
where
    E: serde::de::Error,
{
    type Deserializer = StringCowDeserializer<'de, E>;

    fn into_deserializer(self) -> Self::Deserializer {
        StringCowDeserializer::<E>::new(self)
    }
}

/// A deserializer for [`StringCow`].
pub struct StringCowDeserializer<'a, E> {
    cow: StringCow<'a>,
    _marker: std::marker::PhantomData<E>,
}

impl<'a, E> StringCowDeserializer<'a, E> {
    /// Creates a new [`StringCowDeserializer`].
    pub fn new(cow: StringCow<'a>) -> Self {
        StringCowDeserializer {
            cow,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<'de, E> serde::de::Deserializer<'de> for StringCowDeserializer<'de, E>
where
    E: serde::de::Error,
{
    type Error = E;

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.cow {
            StringCow::Ref(slice) => visitor.visit_borrowed_str(slice),
            StringCow::StaticRef(slice) => visitor.visit_borrowed_str(slice),
            StringCow::String(string) => visitor.visit_string(string),
            StringCow::Bytes(bytes) => visitor.visit_str(&bytes),
        }
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use std::fmt::Display;

    use serde::ser::Impossible;
    use serde::{Deserialize, Serialize};

    use crate::StringCow;

    #[test]
    fn serialize() {
        #[derive(Debug)]
        struct TestError;

        impl Display for TestError {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "Test error")
            }
        }

        impl std::error::Error for TestError {}

        impl serde::ser::Error for TestError {
            fn custom<T: std::fmt::Display>(msg: T) -> Self {
                panic!("{}", msg)
            }
        }

        struct TestSerializer;

        impl serde::Serializer for TestSerializer {
            type Error = TestError;
            type Ok = ();
            type SerializeMap = Impossible<(), Self::Error>;
            type SerializeSeq = Impossible<(), Self::Error>;
            type SerializeStruct = Impossible<(), Self::Error>;
            type SerializeStructVariant = Impossible<(), Self::Error>;
            type SerializeTuple = Impossible<(), Self::Error>;
            type SerializeTupleStruct = Impossible<(), Self::Error>;
            type SerializeTupleVariant = Impossible<(), Self::Error>;

            fn serialize_bool(self, _v: bool) -> Result<Self::Ok, Self::Error> {
                panic!("StringCow must be serialized as str")
            }

            fn serialize_i8(self, _v: i8) -> Result<Self::Ok, Self::Error> {
                panic!("StringCow must be serialized as str")
            }

            fn serialize_i16(self, _v: i16) -> Result<Self::Ok, Self::Error> {
                panic!("StringCow must be serialized as str")
            }

            fn serialize_i32(self, _v: i32) -> Result<Self::Ok, Self::Error> {
                panic!("StringCow must be serialized as str")
            }

            fn serialize_i64(self, _v: i64) -> Result<Self::Ok, Self::Error> {
                panic!("StringCow must be serialized as str")
            }

            fn serialize_u8(self, _v: u8) -> Result<Self::Ok, Self::Error> {
                panic!("StringCow must be serialized as str")
            }

            fn serialize_u16(self, _v: u16) -> Result<Self::Ok, Self::Error> {
                panic!("StringCow must be serialized as str")
            }

            fn serialize_u32(self, _v: u32) -> Result<Self::Ok, Self::Error> {
                panic!("StringCow must be serialized as str")
            }

            fn serialize_u64(self, _v: u64) -> Result<Self::Ok, Self::Error> {
                panic!("StringCow must be serialized as str")
            }

            fn serialize_f32(self, _v: f32) -> Result<Self::Ok, Self::Error> {
                panic!("StringCow must be serialized as str")
            }

            fn serialize_f64(self, _v: f64) -> Result<Self::Ok, Self::Error> {
                panic!("StringCow must be serialized as str")
            }

            fn serialize_char(self, _v: char) -> Result<Self::Ok, Self::Error> {
                panic!("StringCow must be serialized as str")
            }

            fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
                assert_eq!(v, "hello");
                Ok(())
            }

            fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
                panic!("StringCow must be serialized as str")
            }

            fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
                panic!("StringCow must be serialized as str")
            }

            fn serialize_some<T>(self, _value: &T) -> Result<Self::Ok, Self::Error>
            where
                T: serde::Serialize + ?Sized,
            {
                panic!("StringCow must be serialized as str")
            }

            fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
                panic!("StringCow must be serialized as str")
            }

            fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
                panic!("StringCow must be serialized as str")
            }

            fn serialize_unit_variant(
                self,
                _name: &'static str,
                _variant_index: u32,
                _variant: &'static str,
            ) -> Result<Self::Ok, Self::Error> {
                panic!("StringCow must be serialized as str")
            }

            fn serialize_newtype_variant<T>(
                self,
                _name: &'static str,
                _variant_index: u32,
                _variant: &'static str,
                _value: &T,
            ) -> Result<Self::Ok, Self::Error>
            where
                T: serde::Serialize + ?Sized,
            {
                panic!("StringCow must be serialized as str")
            }

            fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
                panic!("StringCow must be serialized as str")
            }

            fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
                panic!("StringCow must be serialized as str")
            }

            fn serialize_tuple_struct(
                self,
                _name: &'static str,
                _len: usize,
            ) -> Result<Self::SerializeTupleStruct, Self::Error> {
                panic!("StringCow must be serialized as str")
            }

            fn serialize_tuple_variant(
                self,
                _name: &'static str,
                _variant_index: u32,
                _variant: &'static str,
                _len: usize,
            ) -> Result<Self::SerializeTupleVariant, Self::Error> {
                panic!("StringCow must be serialized as str")
            }

            fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
                panic!("StringCow must be serialized as str")
            }

            fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct, Self::Error> {
                panic!("StringCow must be serialized as str")
            }

            fn serialize_struct_variant(
                self,
                _name: &'static str,
                _variant_index: u32,
                _variant: &'static str,
                _len: usize,
            ) -> Result<Self::SerializeStructVariant, Self::Error> {
                panic!("StringCow must be serialized as str")
            }

            fn serialize_newtype_struct<T>(self, _name: &'static str, _value: &T) -> Result<Self::Ok, Self::Error>
            where
                T: ?Sized + serde::Serialize,
            {
                panic!("StringCow must be serialized as str")
            }
        }

        let cow = StringCow::from_ref("hello");
        let serializer = TestSerializer;
        cow.serialize(serializer).expect("serialization failed");
    }

    #[test]
    fn deserialize() {
        #[derive(Debug)]
        struct TestError;

        impl Display for TestError {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "Test error")
            }
        }

        impl std::error::Error for TestError {}

        impl serde::de::Error for TestError {
            fn custom<T: std::fmt::Display>(msg: T) -> Self {
                assert_eq!(msg.to_string(), "invalid type: Option value, expected a string");
                Self
            }
        }

        enum Mode {
            Str,
            String,
            BorrowedStr,
            None,
        }

        struct TestDeserializer {
            mode: Mode,
        }

        impl<'de> serde::Deserializer<'de> for TestDeserializer {
            type Error = TestError;

            serde::forward_to_deserialize_any! {
                bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes byte_buf
                option unit unit_struct newtype_struct seq tuple tuple_struct
                map struct enum identifier ignored_any
            }

            fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
            where
                V: serde::de::Visitor<'de>,
            {
                match self.mode {
                    Mode::Str => visitor.visit_str("hello"),
                    Mode::String => visitor.visit_string("hello".to_owned()),
                    Mode::BorrowedStr => visitor.visit_borrowed_str("hello"),
                    Mode::None => visitor.visit_none(),
                }
            }
        }

        fn test_de(de: TestDeserializer) {
            let cow = StringCow::deserialize(de).expect("deserialization failed");
            assert_eq!(cow.as_str(), "hello");
        }

        test_de(TestDeserializer { mode: Mode::Str });
        test_de(TestDeserializer { mode: Mode::String });
        test_de(TestDeserializer { mode: Mode::BorrowedStr });

        StringCow::deserialize(TestDeserializer { mode: Mode::None }).expect_err("deserialization should fail");
    }
}
