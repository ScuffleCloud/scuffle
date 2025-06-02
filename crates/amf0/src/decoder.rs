//! AMF0 decoder

use std::io;

use byteorder::{BigEndian, ReadBytesExt};
use num_traits::FromPrimitive;
use scuffle_bytes_util::StringCow;
use scuffle_bytes_util::zero_copy::ZeroCopyReader;

use crate::{Amf0Error, Amf0Marker, Amf0Object, Amf0Value};

/// AMF0 decoder.
///
/// Provides various functions to decode different types of AMF0 values.
#[derive(Debug, Clone)]
pub struct Amf0Decoder<R> {
    pub(crate) reader: R,
    pub(crate) next_marker: Option<Amf0Marker>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ObjectHeader<'a> {
    Object,
    TypedObject { name: StringCow<'a> },
    EcmaArray { size: u32 },
}

impl<B> Amf0Decoder<scuffle_bytes_util::zero_copy::BytesBuf<B>>
where
    B: bytes::Buf,
{
    /// Create a new deserializer from a buffer implementing [`bytes::Buf`].
    pub fn from_buf(buf: B) -> Self {
        Self {
            reader: buf.into(),
            next_marker: None,
        }
    }
}

impl<R> Amf0Decoder<scuffle_bytes_util::zero_copy::IoRead<R>>
where
    R: std::io::Read,
{
    /// Create a new deserializer from a reader implementing [`std::io::Read`].
    pub fn from_reader(reader: R) -> Self {
        Self {
            reader: reader.into(),
            next_marker: None,
        }
    }
}

impl<'a> Amf0Decoder<scuffle_bytes_util::zero_copy::Slice<'a>> {
    /// Create a new deserializer from a byte slice.
    pub fn from_slice(slice: &'a [u8]) -> Amf0Decoder<scuffle_bytes_util::zero_copy::Slice<'a>> {
        Self {
            reader: slice.into(),
            next_marker: None,
        }
    }
}

impl<'a, R> Amf0Decoder<R>
where
    R: ZeroCopyReader<'a>,
{
    /// Decode a [`Amf0Value`] from the buffer.
    pub fn decode_value(&mut self) -> Result<Amf0Value<'a>, Amf0Error> {
        let marker = self.peek_marker()?;

        match marker {
            Amf0Marker::Boolean => self.decode_boolean().map(Into::into),
            Amf0Marker::Number | Amf0Marker::Date => self.decode_number().map(Into::into),
            Amf0Marker::String | Amf0Marker::LongString | Amf0Marker::XmlDocument => self.decode_string().map(Into::into),
            Amf0Marker::Null | Amf0Marker::Undefined => self.decode_null().map(|_| Amf0Value::Null),
            Amf0Marker::Object | Amf0Marker::TypedObject | Amf0Marker::EcmaArray => self.decode_object().map(Into::into),
            Amf0Marker::StrictArray => self.decode_strict_array().map(Into::into),
            _ => Err(Amf0Error::UnsupportedMarker(marker)),
        }
    }

    /// Decode all values from the buffer until the end.
    pub fn decode_all(&mut self) -> Result<Vec<Amf0Value<'a>>, Amf0Error> {
        let mut values = Vec::new();

        while self.has_remaining()? {
            values.push(self.decode_value()?);
        }

        Ok(values)
    }

    /// Convert the decoder into an iterator over the values in the buffer.
    pub fn stream(&mut self) -> Amf0DecoderStream<'_, 'a, R> {
        Amf0DecoderStream {
            decoder: self,
            _marker: std::marker::PhantomData,
        }
    }

    /// Check if there are any values left in the buffer.
    pub fn has_remaining(&mut self) -> Result<bool, Amf0Error> {
        match self.peek_marker() {
            Ok(_) => Ok(true),
            Err(Amf0Error::Io(e)) if e.kind() == io::ErrorKind::UnexpectedEof => Ok(false),
            Err(err) => Err(err),
        }
    }

    /// Peek the next marker in the buffer without consuming it.
    pub fn peek_marker(&mut self) -> Result<Amf0Marker, Amf0Error> {
        let marker = self.read_marker()?;
        // Buffer the marker for the next read
        self.next_marker = Some(marker);

        Ok(marker)
    }

    fn read_marker(&mut self) -> Result<Amf0Marker, Amf0Error> {
        if let Some(marker) = self.next_marker.take() {
            return Ok(marker);
        }

        let marker = self.reader.as_std().read_u8()?;
        let marker = Amf0Marker::from_u8(marker).ok_or(Amf0Error::UnknownMarker(marker))?;
        Ok(marker)
    }

    fn expect_marker(&mut self, expect: &'static [Amf0Marker]) -> Result<Amf0Marker, Amf0Error> {
        let marker = self.read_marker()?;

        if !expect.contains(&marker) {
            Err(Amf0Error::UnexpectedType {
                expected: expect,
                got: marker,
            })
        } else {
            Ok(marker)
        }
    }

    /// Decode a number from the buffer.
    pub fn decode_number(&mut self) -> Result<f64, Amf0Error> {
        let marker = self.expect_marker(&[Amf0Marker::Number, Amf0Marker::Date])?;

        let number = self.reader.as_std().read_f64::<BigEndian>()?;

        if marker == Amf0Marker::Date {
            // Skip the timezone
            self.reader.as_std().read_i16::<BigEndian>()?;
        }

        Ok(number)
    }

    /// Decode a boolean from the buffer.
    pub fn decode_boolean(&mut self) -> Result<bool, Amf0Error> {
        self.expect_marker(&[Amf0Marker::Boolean])?;
        let value = self.reader.as_std().read_u8()?;
        Ok(value != 0)
    }

    pub(crate) fn decode_normal_string(&mut self) -> Result<StringCow<'a>, Amf0Error> {
        let len = self.reader.as_std().read_u16::<BigEndian>()? as usize;

        let bytes = self.reader.try_read(len)?;
        Ok(StringCow::from_bytes(bytes.into_bytes().try_into()?))
    }

    /// Decode a string from the buffer.
    ///
    /// This function can decode both normal strings and long strings.
    pub fn decode_string(&mut self) -> Result<StringCow<'a>, Amf0Error> {
        let marker = self.expect_marker(&[Amf0Marker::String, Amf0Marker::LongString, Amf0Marker::XmlDocument])?;

        let len = if marker == Amf0Marker::String {
            self.reader.as_std().read_u16::<BigEndian>()? as usize
        } else {
            // LongString or XmlDocument
            self.reader.as_std().read_u32::<BigEndian>()? as usize
        };

        let bytes = self.reader.try_read(len)?;
        Ok(StringCow::from_bytes(bytes.into_bytes().try_into()?))
    }

    /// Decode a null value from the buffer.
    ///
    /// This function can also decode undefined values.
    pub fn decode_null(&mut self) -> Result<(), Amf0Error> {
        self.expect_marker(&[Amf0Marker::Null, Amf0Marker::Undefined])?;
        Ok(())
    }

    /// Deserialize a value from the buffer using [serde].
    #[cfg(feature = "serde")]
    pub fn deserialize<T>(&mut self) -> Result<T, Amf0Error>
    where
        T: serde::de::Deserialize<'a>,
    {
        T::deserialize(self)
    }

    /// Deserialize a stream of values from the buffer using [serde].
    #[cfg(feature = "serde")]
    pub fn deserialize_stream<T>(&mut self) -> crate::de::Amf0DeserializerStream<'_, R, T>
    where
        T: serde::de::Deserialize<'a>,
    {
        crate::de::Amf0DeserializerStream::new(self)
    }

    // --- Object and Ecma array ---

    pub(crate) fn decode_object_header(&mut self) -> Result<ObjectHeader<'a>, Amf0Error> {
        let marker = self.expect_marker(&[Amf0Marker::Object, Amf0Marker::TypedObject, Amf0Marker::EcmaArray])?;

        if marker == Amf0Marker::Object {
            Ok(ObjectHeader::Object)
        } else if marker == Amf0Marker::TypedObject {
            let name = self.decode_normal_string()?;
            Ok(ObjectHeader::TypedObject { name })
        } else {
            // EcmaArray
            let size = self.reader.as_std().read_u32::<BigEndian>()?;
            Ok(ObjectHeader::EcmaArray { size })
        }
    }

    pub(crate) fn decode_object_key(&mut self) -> Result<Option<StringCow<'a>>, Amf0Error> {
        // Object keys are not preceeded with a marker and are always normal strings
        let key = self.decode_normal_string()?;

        // The object end marker is preceeded by an empty string
        if key.as_str().is_empty() {
            // Check if the next marker is an object end marker
            if self.peek_marker()? == Amf0Marker::ObjectEnd {
                // Clear the next marker buffer
                self.next_marker = None;

                return Ok(None);
            }
        }

        Ok(Some(key))
    }

    /// Decode an object from the buffer.
    ///
    /// This function can decode normal objects, typed objects and ECMA arrays.
    pub fn decode_object(&mut self) -> Result<Amf0Object<'a>, Amf0Error> {
        let header = self.decode_object_header()?;

        match header {
            ObjectHeader::Object | ObjectHeader::TypedObject { .. } => {
                let mut object = Amf0Object::new();

                while let Some(key) = self.decode_object_key()? {
                    let value = self.decode_value()?;
                    object.insert(key, value);
                }

                Ok(object)
            }
            ObjectHeader::EcmaArray { size } => {
                let mut object = Amf0Object::new();

                for _ in 0..size {
                    // Object keys are not preceeded with a marker and are always normal strings
                    let key = self.decode_normal_string()?;
                    let value = self.decode_value()?;
                    object.insert(key, value);
                }

                // There might be an object end marker after the last key
                if self.has_remaining()? && self.peek_marker()? == Amf0Marker::ObjectEnd {
                    // Clear the next marker buffer
                    self.next_marker = None;
                }

                Ok(object)
            }
        }
    }

    // --- Strict array ---

    pub(crate) fn decode_strict_array_header(&mut self) -> Result<u32, Amf0Error> {
        self.expect_marker(&[Amf0Marker::StrictArray])?;
        let size = self.reader.as_std().read_u32::<BigEndian>()?;

        Ok(size)
    }

    /// Decode a strict array from the buffer.
    pub fn decode_strict_array(&mut self) -> Result<Vec<Amf0Value<'a>>, Amf0Error> {
        let size = self.decode_strict_array_header()? as usize;

        let mut array = Vec::with_capacity(size);

        for _ in 0..size {
            let value = self.decode_value()?;
            array.push(value);
        }

        Ok(array)
    }
}

/// An iterator over the values in the buffer.
///
/// Yields values of type [`Amf0Value`] until the end of the buffer is reached.
#[must_use = "Iterators are lazy and do nothing unless consumed"]
pub struct Amf0DecoderStream<'a, 'de, R> {
    decoder: &'a mut Amf0Decoder<R>,
    _marker: std::marker::PhantomData<&'de ()>,
}

impl<'de, R: ZeroCopyReader<'de>> Iterator for Amf0DecoderStream<'_, 'de, R> {
    type Item = Result<Amf0Value<'de>, Amf0Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.decoder.has_remaining() {
            Ok(true) => Some(self.decoder.decode_value()),
            Ok(false) => None,
            Err(err) => Some(Err(err)),
        }
    }
}

impl<'de, R> std::iter::FusedIterator for Amf0DecoderStream<'_, 'de, R> where R: ZeroCopyReader<'de> {}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use scuffle_bytes_util::StringCow;

    use super::Amf0Decoder;
    use crate::{Amf0Marker, Amf0Value};

    #[test]
    fn strict_array() {
        #[rustfmt::skip]
        let bytes = [
            Amf0Marker::StrictArray as u8,
            0, 0, 0, 2, // size
            Amf0Marker::String as u8,
            0, 3, b'v', b'a', b'l', // value
            Amf0Marker::Boolean as u8,
            1, // value
        ];

        let mut decoder = Amf0Decoder::from_slice(&bytes);
        let array = decoder.decode_strict_array().unwrap();
        assert_eq!(array.len(), 2);
        assert_eq!(array[0], Amf0Value::String("val".into()));
        assert_eq!(array[1], Amf0Value::Boolean(true));
    }

    #[test]
    fn ecma_array() {
        #[rustfmt::skip]
        let bytes = [
            Amf0Marker::EcmaArray as u8,
            0, 0, 0, 2, // size
            0, 3, b'a', b'b', b'c', // key
            Amf0Marker::String as u8,
            0, 3, b'v', b'a', b'l', // value
            0, 4, b'd', b'e', b'f', b'g', // key
            Amf0Marker::Boolean as u8,
            1, // value
        ];

        let mut decoder = Amf0Decoder::from_slice(&bytes);
        let object = decoder.decode_object().unwrap();
        assert_eq!(object.len(), 2);
        assert_eq!(
            *object.get(&StringCow::from_static("abc")).unwrap(),
            Amf0Value::String("val".into())
        );
        assert_eq!(
            *object.get(&StringCow::from_static("defg")).unwrap(),
            Amf0Value::Boolean(true)
        );
    }

    #[test]
    fn decoder_stream() {
        #[rustfmt::skip]
        let bytes = [
            Amf0Marker::Boolean as u8,
            1, // value
            Amf0Marker::String as u8,
            0, 3, b'a', b'b', b'c', // value
            Amf0Marker::Null as u8,
        ];

        let mut decoder = Amf0Decoder::from_slice(&bytes);
        let mut stream = decoder.stream();
        assert_eq!(stream.next().unwrap().unwrap(), Amf0Value::Boolean(true));
        assert_eq!(stream.next().unwrap().unwrap(), Amf0Value::String("abc".into()));
        assert_eq!(stream.next().unwrap().unwrap(), Amf0Value::Null);
        assert!(stream.next().is_none());
    }
}
