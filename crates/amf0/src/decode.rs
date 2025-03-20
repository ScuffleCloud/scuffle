use std::borrow::Cow;
use std::io::{Cursor, Seek, SeekFrom};

use byteorder::{BigEndian, ReadBytesExt};
use num_traits::FromPrimitive;

use super::{Amf0Marker, Amf0ReadError, Amf0Value};

/// An AMF0 Decoder.
///
/// This decoder takes a reference to a byte slice and reads the AMF0 data from
/// it. All returned objects are references to the original byte slice. Making
/// it very cheap to use.
pub struct Amf0Decoder<'a> {
    cursor: Cursor<&'a [u8]>,
}

impl<'a> Amf0Decoder<'a> {
    /// Create a new AMF0 decoder.
    pub const fn new(buff: &'a [u8]) -> Self {
        Self {
            cursor: Cursor::new(buff),
        }
    }

    /// Check if the decoder has reached the end of the AMF0 data.
    pub const fn is_empty(&self) -> bool {
        self.cursor.get_ref().len() == self.cursor.position() as usize
    }

    fn read_bytes(&mut self, len: usize) -> Result<&'a [u8], Amf0ReadError> {
        let pos = self.cursor.position();
        self.cursor.seek(SeekFrom::Current(len as i64))?;
        Ok(&self.cursor.get_ref()[pos as usize..pos as usize + len])
    }

    /// Read all the encoded values from the decoder.
    pub fn decode_all(&mut self) -> Result<Vec<Amf0Value<'a>>, Amf0ReadError> {
        let mut results = vec![];

        while !self.is_empty() {
            results.push(self.decode()?);
        }

        Ok(results)
    }

    /// Read the next encoded value from the decoder.
    pub fn decode(&mut self) -> Result<Amf0Value<'a>, Amf0ReadError> {
        let marker = self.cursor.read_u8()?;
        let marker = Amf0Marker::from_u8(marker).ok_or(Amf0ReadError::UnknownMarker(marker))?;

        match marker {
            Amf0Marker::Number => Ok(Amf0Value::Number(self.read_number()?)),
            Amf0Marker::Boolean => Ok(Amf0Value::Boolean(self.read_bool()?)),
            Amf0Marker::String => Ok(Amf0Value::String(self.read_string()?)),
            Amf0Marker::Object => Ok(Amf0Value::Object(self.read_object()?.into())),
            Amf0Marker::Null => Ok(Amf0Value::Null),
            Amf0Marker::EcmaArray => Ok(Amf0Value::Object(self.read_ecma_array()?.into())),
            Amf0Marker::LongString => Ok(Amf0Value::LongString(self.read_long_string()?)),
            _ => Err(Amf0ReadError::UnsupportedType(marker)),
        }
    }

    /// Read the next encoded value from the decoder and check if it matches the
    /// specified marker.
    pub fn decode_with_type(&mut self, specified_marker: Amf0Marker) -> Result<Amf0Value<'a>, Amf0ReadError> {
        let marker = self.cursor.read_u8()?;
        self.cursor.seek(SeekFrom::Current(-1))?; // seek back to the original position

        let marker = Amf0Marker::from_u8(marker).ok_or(Amf0ReadError::UnknownMarker(marker))?;
        if marker != specified_marker {
            return Err(Amf0ReadError::WrongType {
                expected: specified_marker,
                got: marker,
            });
        }

        self.decode()
    }

    fn read_number(&mut self) -> Result<f64, Amf0ReadError> {
        Ok(self.cursor.read_f64::<BigEndian>()?)
    }

    fn read_bool(&mut self) -> Result<bool, Amf0ReadError> {
        Ok(self.cursor.read_u8()? > 0)
    }

    fn read_string(&mut self) -> Result<Cow<'a, str>, Amf0ReadError> {
        let l = self.cursor.read_u16::<BigEndian>()?;
        let bytes = self.read_bytes(l as usize)?;

        Ok(Cow::Borrowed(std::str::from_utf8(bytes)?))
    }

    fn is_read_object_eof(&mut self) -> Result<bool, Amf0ReadError> {
        let pos = self.cursor.position();
        let marker = self.cursor.read_u24::<BigEndian>().map(Amf0Marker::from_u32);

        match marker {
            Ok(Some(Amf0Marker::ObjectEnd)) => Ok(true),
            _ => {
                self.cursor.seek(SeekFrom::Start(pos))?;
                Ok(false)
            }
        }
    }

    fn read_object(&mut self) -> Result<Vec<(Cow<'a, str>, Amf0Value<'a>)>, Amf0ReadError> {
        let mut properties = Vec::new();

        loop {
            let is_eof = self.is_read_object_eof()?;

            if is_eof {
                break;
            }

            let key = self.read_string()?;
            let val = self.decode()?;

            properties.push((key, val));
        }

        Ok(properties)
    }

    fn read_ecma_array(&mut self) -> Result<Vec<(Cow<'a, str>, Amf0Value<'a>)>, Amf0ReadError> {
        let len = self.cursor.read_u32::<BigEndian>()?;

        let mut properties = Vec::new();

        for _ in 0..len {
            let key = self.read_string()?;
            let val = self.decode()?;
            properties.push((key, val));
        }

        // Sometimes the object end marker is present and sometimes it is not.
        // If it is there just read it, if not then we are done.
        self.is_read_object_eof().ok(); // ignore the result

        Ok(properties)
    }

    fn read_long_string(&mut self) -> Result<Cow<'a, str>, Amf0ReadError> {
        let l = self.cursor.read_u32::<BigEndian>()?;

        let buff = self.read_bytes(l as usize)?;
        let val = std::str::from_utf8(buff)?;

        Ok(Cow::Borrowed(val))
    }
}

impl<'a> Iterator for Amf0Decoder<'a> {
    type Item = Result<Amf0Value<'a>, Amf0ReadError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_empty() {
            return None;
        }

        Some(self.decode())
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn test_reader_bool() {
        let amf0_bool = vec![0x01, 0x01]; // true
        let mut amf_reader = Amf0Decoder::new(&amf0_bool);
        let value = amf_reader.decode_with_type(Amf0Marker::Boolean).unwrap();
        assert_eq!(value, Amf0Value::Boolean(true));
    }

    #[test]
    fn test_reader_number() {
        let mut amf0_number = vec![0x00];
        amf0_number.extend_from_slice(&772.161_f64.to_be_bytes());

        let mut amf_reader = Amf0Decoder::new(&amf0_number);
        let value = amf_reader.decode_with_type(Amf0Marker::Number).unwrap();
        assert_eq!(value, Amf0Value::Number(772.161));
    }

    #[test]
    fn test_reader_string() {
        let mut amf0_string = vec![0x02, 0x00, 0x0b]; // 11 bytes
        amf0_string.extend_from_slice(b"Hello World");

        let mut amf_reader = Amf0Decoder::new(&amf0_string);
        let value = amf_reader.decode_with_type(Amf0Marker::String).unwrap();
        assert_eq!(value, Amf0Value::String(Cow::Borrowed("Hello World")));
    }

    #[test]
    fn test_reader_long_string() {
        let mut amf0_string = vec![0x0c, 0x00, 0x00, 0x00, 0x0b]; // 11 bytes
        amf0_string.extend_from_slice(b"Hello World");

        let mut amf_reader = Amf0Decoder::new(&amf0_string);
        let value = amf_reader.decode_with_type(Amf0Marker::LongString).unwrap();
        assert_eq!(value, Amf0Value::LongString(Cow::Borrowed("Hello World")));
    }

    #[test]
    fn test_reader_object() {
        let mut amf0_object = vec![0x03, 0x00, 0x04]; // 1 property with 4 bytes
        amf0_object.extend_from_slice(b"test");
        amf0_object.extend_from_slice(&[0x05]); // null
        amf0_object.extend_from_slice(&[0x00, 0x00, 0x09]); // object end (0x00 0x00 0x09)

        let mut amf_reader = Amf0Decoder::new(&amf0_object);
        let value = amf_reader.decode_with_type(Amf0Marker::Object).unwrap();

        assert_eq!(value, Amf0Value::Object(vec![("test".into(), Amf0Value::Null)].into()));
    }

    #[test]
    fn test_reader_ecma_array() {
        let mut amf0_object = vec![0x08, 0x00, 0x00, 0x00, 0x01]; // 1 property
        amf0_object.extend_from_slice(&[0x00, 0x04]); // 4 bytes
        amf0_object.extend_from_slice(b"test");
        amf0_object.extend_from_slice(&[0x05]); // null

        let mut amf_reader = Amf0Decoder::new(&amf0_object);
        let value = amf_reader.decode_with_type(Amf0Marker::EcmaArray).unwrap();

        assert_eq!(value, Amf0Value::Object(vec![("test".into(), Amf0Value::Null)].into()));
    }

    #[test]
    fn test_reader_multi_value() {
        let mut amf0_multi = vec![0x00];
        amf0_multi.extend_from_slice(&772.161_f64.to_be_bytes());
        amf0_multi.extend_from_slice(&[0x01, 0x01]); // true
        amf0_multi.extend_from_slice(&[0x02, 0x00, 0x0b]); // 11 bytes
        amf0_multi.extend_from_slice(b"Hello World");
        amf0_multi.extend_from_slice(&[0x03, 0x00, 0x04]); // 1 property with 4 bytes
        amf0_multi.extend_from_slice(b"test");
        amf0_multi.extend_from_slice(&[0x05]); // null
        amf0_multi.extend_from_slice(&[0x00, 0x00, 0x09]); // object end (0x00 0x00 0x09)

        let mut amf_reader = Amf0Decoder::new(&amf0_multi);
        let values = amf_reader.decode_all().unwrap();

        assert_eq!(values.len(), 4);

        assert_eq!(values[0], Amf0Value::Number(772.161));
        assert_eq!(values[1], Amf0Value::Boolean(true));
        assert_eq!(values[2], Amf0Value::String(Cow::Borrowed("Hello World")));
        assert_eq!(values[3], Amf0Value::Object(vec![("test".into(), Amf0Value::Null)].into()));
    }

    #[test]
    fn test_reader_iterator() {
        let mut amf0_multi = vec![0x00];
        amf0_multi.extend_from_slice(&772.161_f64.to_be_bytes());
        amf0_multi.extend_from_slice(&[0x01, 0x01]); // true
        amf0_multi.extend_from_slice(&[0x02, 0x00, 0x0b]); // 11 bytes
        amf0_multi.extend_from_slice(b"Hello World");

        let amf_reader = Amf0Decoder::new(&amf0_multi);
        let values = amf_reader.collect::<Result<Vec<_>, _>>().unwrap();

        assert_eq!(values.len(), 3);

        assert_eq!(values[0], Amf0Value::Number(772.161));
        assert_eq!(values[1], Amf0Value::Boolean(true));
        assert_eq!(values[2], Amf0Value::String(Cow::Borrowed("Hello World")));
    }

    #[test]
    fn test_reader_invalid_marker() {
        let amf0_unsupported_marker = vec![Amf0Marker::Unsupported as u8];
        let mut amf_reader = Amf0Decoder::new(&amf0_unsupported_marker);
        let result = amf_reader.decode();

        assert!(matches!(result, Err(Amf0ReadError::UnsupportedType(Amf0Marker::Unsupported))));
    }
}
