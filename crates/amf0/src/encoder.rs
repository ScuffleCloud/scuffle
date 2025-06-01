//! AMF0 encoder

use std::borrow::Borrow;
use std::io;

use byteorder::{BigEndian, WriteBytesExt};

use crate::{Amf0Error, Amf0Marker, Amf0Object, Amf0Value};

/// AMF0 encoder.
///
/// Provides various functions to encode different types of AMF0 values into a writer.
#[derive(Debug)]
pub struct Amf0Encoder<W> {
    writer: W,
}

impl<W> Amf0Encoder<W> {
    /// Create a new encoder from a writer.
    pub fn new(writer: W) -> Self {
        Amf0Encoder { writer }
    }
}

impl<W> Amf0Encoder<W>
where
    W: io::Write,
{
    /// Encode a [`bool`] as a AMF0 boolean value.
    pub fn encode_boolean(&mut self, value: bool) -> Result<(), Amf0Error> {
        self.writer.write_u8(Amf0Marker::Boolean as u8)?;
        self.writer.write_u8(value as u8)?;
        Ok(())
    }

    /// Encode a [`f64`] as a AMF0 number value.
    pub fn encode_number(&mut self, value: f64) -> Result<(), Amf0Error> {
        self.writer.write_u8(Amf0Marker::Number as u8)?;
        self.writer.write_f64::<BigEndian>(value)?;
        Ok(())
    }

    /// Encode a [`&str`](str) as a AMF0 string value.
    ///
    /// This function decides based on the length of the given string slice whether to use a normal string or a long string.
    pub fn encode_string(&mut self, value: &str) -> Result<(), Amf0Error> {
        let len = value.len();

        if len <= (u16::MAX as usize) {
            // Normal string
            self.writer.write_u8(Amf0Marker::String as u8)?;
            self.writer.write_u16::<BigEndian>(len as u16)?;
            self.writer.write_all(value.as_bytes())?;
        } else {
            // Long string

            // This try_into fails if the length is greater than u32::MAX
            let len: u32 = len.try_into()?;

            self.writer.write_u8(Amf0Marker::LongString as u8)?;
            self.writer.write_u32::<BigEndian>(len)?;
            self.writer.write_all(value.as_bytes())?;
        }

        Ok(())
    }

    /// Encode AMF0 Null value.
    pub fn encode_null(&mut self) -> Result<(), Amf0Error> {
        self.writer.write_u8(Amf0Marker::Null as u8)?;
        Ok(())
    }

    /// Encode AMF0 Undefined value.
    pub fn encode_undefined(&mut self) -> Result<(), Amf0Error> {
        self.writer.write_u8(Amf0Marker::Undefined as u8)?;
        Ok(())
    }

    pub(crate) fn encode_array_header(&mut self, len: u32) -> Result<(), Amf0Error> {
        self.writer.write_u8(Amf0Marker::StrictArray as u8)?;
        self.writer.write_u32::<BigEndian>(len)?;
        Ok(())
    }

    /// Encode an [`Amf0Array`] as an AMF0 StrictArray value.
    pub fn encode_array<'a, I, B>(&mut self, values: I) -> Result<(), Amf0Error>
    where
        B: Borrow<Amf0Value<'a>>,
        I: IntoIterator<Item = B>,
        I::IntoIter: ExactSizeIterator,
    {
        let iter = values.into_iter();
        self.encode_array_header(iter.len().try_into()?)?;

        for value in iter {
            value.borrow().encode(self)?;
        }

        Ok(())
    }

    pub(crate) fn encode_object_header(&mut self) -> Result<(), Amf0Error> {
        self.writer.write_u8(Amf0Marker::Object as u8)?;
        Ok(())
    }

    pub(crate) fn encode_object_key(&mut self, key: &str) -> Result<(), Amf0Error> {
        self.writer.write_u16::<BigEndian>(key.len().try_into()?)?;
        self.writer.write_all(key.as_bytes())?;
        Ok(())
    }

    pub(crate) fn encode_object_trailer(&mut self) -> Result<(), Amf0Error> {
        self.writer.write_u24::<BigEndian>(Amf0Marker::ObjectEnd as u32)?;
        Ok(())
    }

    /// Encode an [`Amf0Object`] as an AMF0 Object value.
    pub fn encode_object(&mut self, values: &Amf0Object) -> Result<(), Amf0Error> {
        self.encode_object_header()?;

        for (key, value) in values.iter() {
            self.encode_object_key(key.as_str())?;
            value.encode(self)?;
        }

        self.encode_object_trailer()?;

        Ok(())
    }

    /// Encode a given value using [serde].
    #[cfg(feature = "serde")]
    pub fn serialize<T>(&mut self, value: T) -> Result<(), Amf0Error>
    where
        T: serde::Serialize,
    {
        value.serialize(self)?;
        Ok(())
    }
}
