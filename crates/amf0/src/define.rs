use std::borrow::Cow;

use num_derive::FromPrimitive;

use crate::Amf0ReadError;

/// AMF0 marker types.
/// Defined in amf0_spec_121207.pdf section 2.1
#[derive(Debug, PartialEq, Eq, Clone, Copy, FromPrimitive)]
#[repr(u8)]
pub enum Amf0Marker {
    /// number-marker
    Number = 0x00,
    /// boolean-marker
    Boolean = 0x01,
    /// string-marker
    String = 0x02,
    /// object-marker
    Object = 0x03,
    /// movieclip-marker
    ///
    /// reserved, not supported
    MovieClipMarker = 0x04,
    /// null-marker
    Null = 0x05,
    /// undefined-marker
    Undefined = 0x06,
    /// reference-marker
    Reference = 0x07,
    /// ecma-array-marker
    EcmaArray = 0x08,
    /// object-end-marker
    ObjectEnd = 0x09,
    /// strict-array-marker
    StrictArray = 0x0a,
    /// date-marker
    Date = 0x0b,
    /// long-string-marker
    LongString = 0x0c,
    /// unsupported-marker
    Unsupported = 0x0d,
    /// recordset-marker
    ///
    /// reserved, not supported
    Recordset = 0x0e,
    /// xml-document-marker
    XmlDocument = 0x0f,
    /// typed-object-marker
    TypedObject = 0x10,
    /// avmplus-object-marker
    ///
    /// AMF3 marker
    AVMPlusObject = 0x11,
}

/// AMF0 object type.
pub type Amf0Object<'a> = Cow<'a, [(Cow<'a, str>, Amf0Value<'a>)]>;

/// AMF0 value types.
/// Defined in amf0_spec_121207.pdf section 2.2-2.14
#[derive(PartialEq, Clone, Debug)]
pub enum Amf0Value<'a> {
    /// Number Type defined section 2.2
    Number(f64),
    /// Boolean Type defined section 2.3
    Boolean(bool),
    /// String Type defined section 2.4
    String(Cow<'a, str>),
    /// Object Type defined section 2.5
    Object(Amf0Object<'a>),
    /// Null Type defined section 2.7
    Null,
    /// Undefined Type defined section 2.8
    ObjectEnd,
    /// LongString Type defined section 2.14
    LongString(Cow<'a, str>),
}

impl From<f64> for Amf0Value<'_> {
    fn from(value: f64) -> Self {
        Amf0Value::Number(value)
    }
}

impl From<bool> for Amf0Value<'_> {
    fn from(value: bool) -> Self {
        Amf0Value::Boolean(value)
    }
}

impl<'a> From<Cow<'a, str>> for Amf0Value<'a> {
    fn from(value: Cow<'a, str>) -> Self {
        // Check if the string is too long to fit in a normal amf0 string (2 bytes length)
        if value.len() > u16::MAX as usize {
            Amf0Value::LongString(value)
        } else {
            Amf0Value::String(value)
        }
    }
}

impl<'a> From<Amf0Object<'a>> for Amf0Value<'a> {
    fn from(value: Amf0Object<'a>) -> Self {
        Amf0Value::Object(value)
    }
}

impl Amf0Value<'_> {
    /// Get the marker of the value.
    pub fn marker(&self) -> Amf0Marker {
        match self {
            Self::Boolean(_) => Amf0Marker::Boolean,
            Self::Number(_) => Amf0Marker::Number,
            Self::String(_) => Amf0Marker::String,
            Self::Object(_) => Amf0Marker::Object,
            Self::Null => Amf0Marker::Null,
            Self::ObjectEnd => Amf0Marker::ObjectEnd,
            Self::LongString(_) => Amf0Marker::LongString,
        }
    }

    /// Get the owned value.
    pub fn to_owned(&self) -> Amf0Value<'static> {
        match self {
            Self::String(s) => Amf0Value::String(Cow::Owned(s.to_string())),
            Self::LongString(s) => Amf0Value::LongString(Cow::Owned(s.to_string())),
            Self::Object(o) => Amf0Value::Object(o.iter().map(|(k, v)| (Cow::Owned(k.to_string()), v.to_owned())).collect()),
            Self::Number(n) => Amf0Value::Number(*n),
            Self::Boolean(b) => Amf0Value::Boolean(*b),
            Self::Null => Amf0Value::Null,
            Self::ObjectEnd => Amf0Value::ObjectEnd,
        }
    }

    /// Get the value as a number.
    ///
    /// Returns [`Amf0ReadError::WrongType`] if the value is not a number.
    pub fn as_number(&self) -> Result<f64, Amf0ReadError> {
        match self {
            Self::Number(n) => Ok(*n),
            _ => Err(Amf0ReadError::WrongType {
                expected: Amf0Marker::Number,
                got: self.marker(),
            }),
        }
    }

    /// Get the value as a boolean.
    ///
    /// Returns [`Amf0ReadError::WrongType`] if the value is not a boolean.
    pub fn as_boolean(&self) -> Result<bool, Amf0ReadError> {
        match self {
            Self::Boolean(b) => Ok(*b),
            _ => Err(Amf0ReadError::WrongType {
                expected: Amf0Marker::Boolean,
                got: self.marker(),
            }),
        }
    }
}

impl<'a> Amf0Value<'a> {
    /// Get the value as a string.
    ///
    /// Returns [`Amf0ReadError::WrongType`] if the value is not a string.
    pub fn as_string(&self) -> Result<&Cow<'a, str>, Amf0ReadError> {
        match self {
            Self::String(s) => Ok(s),
            Self::LongString(s) => Ok(s),
            _ => Err(Amf0ReadError::WrongType {
                expected: Amf0Marker::String,
                got: self.marker(),
            }),
        }
    }

    /// Get the value as an object.
    ///
    /// Returns [`Amf0ReadError::WrongType`] if the value is not an object.
    pub fn as_object(&self) -> Result<&Amf0Object<'a>, Amf0ReadError> {
        match self {
            Self::Object(o) => Ok(o),
            _ => Err(Amf0ReadError::WrongType {
                expected: Amf0Marker::Object,
                got: self.marker(),
            }),
        }
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use num_traits::FromPrimitive;

    use super::*;

    #[test]
    fn test_marker() {
        let cases = [
            (Amf0Value::Number(1.0), Amf0Marker::Number),
            (Amf0Value::Boolean(true), Amf0Marker::Boolean),
            (Amf0Value::String(Cow::Borrowed("test")), Amf0Marker::String),
            (
                Amf0Value::Object(Cow::Borrowed(&[(Cow::Borrowed("test"), Amf0Value::Number(1.0))])),
                Amf0Marker::Object,
            ),
            (Amf0Value::Null, Amf0Marker::Null),
            (Amf0Value::ObjectEnd, Amf0Marker::ObjectEnd),
            (Amf0Value::LongString(Cow::Borrowed("test")), Amf0Marker::LongString),
        ];

        for (value, marker) in cases {
            assert_eq!(value.marker(), marker);
        }
    }

    #[test]
    fn test_to_owned() {
        let value = Amf0Value::Object(Cow::Borrowed(&[(
            Cow::Borrowed("test"),
            Amf0Value::LongString(Cow::Borrowed("test")),
        )]));
        let owned = value.to_owned();
        assert_eq!(
            owned,
            Amf0Value::Object(Cow::Owned(vec![(
                "test".to_string().into(),
                Amf0Value::LongString(Cow::Owned("test".to_string()))
            )]))
        );

        let value = Amf0Value::String(Cow::Borrowed("test"));
        let owned = value.to_owned();
        assert_eq!(owned, Amf0Value::String(Cow::Owned("test".to_string())));

        let value = Amf0Value::Number(1.0);
        let owned = value.to_owned();
        assert_eq!(owned, Amf0Value::Number(1.0));

        let value = Amf0Value::Boolean(true);
        let owned = value.to_owned();
        assert_eq!(owned, Amf0Value::Boolean(true));

        let value = Amf0Value::Null;
        let owned = value.to_owned();
        assert_eq!(owned, Amf0Value::Null);

        let value = Amf0Value::ObjectEnd;
        let owned = value.to_owned();
        assert_eq!(owned, Amf0Value::ObjectEnd);
    }

    #[test]
    fn test_marker_primitive() {
        let cases = [
            (Amf0Marker::Number, 0x00),
            (Amf0Marker::Boolean, 0x01),
            (Amf0Marker::String, 0x02),
            (Amf0Marker::Object, 0x03),
            (Amf0Marker::MovieClipMarker, 0x04),
            (Amf0Marker::Null, 0x05),
            (Amf0Marker::Undefined, 0x06),
            (Amf0Marker::Reference, 0x07),
            (Amf0Marker::EcmaArray, 0x08),
            (Amf0Marker::ObjectEnd, 0x09),
            (Amf0Marker::StrictArray, 0x0a),
            (Amf0Marker::Date, 0x0b),
            (Amf0Marker::LongString, 0x0c),
            (Amf0Marker::Unsupported, 0x0d),
            (Amf0Marker::Recordset, 0x0e),
            (Amf0Marker::XmlDocument, 0x0f),
            (Amf0Marker::TypedObject, 0x10),
            (Amf0Marker::AVMPlusObject, 0x11),
        ];

        for (marker, value) in cases {
            assert_eq!(marker as u8, value);
            assert_eq!(Amf0Marker::from_u8(value), Some(marker));
        }

        assert!(Amf0Marker::from_u8(0x12).is_none());
    }

    #[test]
    fn from_impls() {
        let number = Amf0Value::from(1.0);
        assert_eq!(number, Amf0Value::Number(1.0));

        let boolean = Amf0Value::from(true);
        assert_eq!(boolean, Amf0Value::Boolean(true));

        let string = Amf0Value::from(Cow::Borrowed("test"));
        assert_eq!(string, Amf0Value::String(Cow::Borrowed("test")));

        // long string
        let long_string: Cow<'_, str> = Cow::Owned("a".repeat(u16::MAX as usize + 1));
        let string = Amf0Value::from(long_string.clone());
        assert_eq!(string, Amf0Value::LongString(long_string));

        let object: Amf0Object = vec![(Cow::Borrowed("test"), Amf0Value::Number(1.0))].into();
        let object = Amf0Value::from(object);
        assert_eq!(
            object,
            Amf0Value::Object(Cow::Borrowed(&[(Cow::Borrowed("test"), Amf0Value::Number(1.0))]))
        );
    }

    #[test]
    fn as_functions() {
        let number = Amf0Value::from(1.0);
        assert_eq!(number.as_number().unwrap(), 1.0);
        assert!(matches!(
            number.as_boolean(),
            Err(Amf0ReadError::WrongType {
                expected: Amf0Marker::Boolean,
                got: Amf0Marker::Number
            })
        ));

        let boolean = Amf0Value::from(true);
        assert!(boolean.as_boolean().unwrap());
        assert!(matches!(
            boolean.as_number(),
            Err(Amf0ReadError::WrongType {
                expected: Amf0Marker::Number,
                got: Amf0Marker::Boolean
            })
        ));

        let string = Amf0Value::from(Cow::Borrowed("test"));
        assert_eq!(string.as_string().unwrap(), &Cow::Borrowed("test"));
        assert!(matches!(
            string.as_boolean(),
            Err(Amf0ReadError::WrongType {
                expected: Amf0Marker::Boolean,
                got: Amf0Marker::String
            })
        ));

        // long string
        let long_string: Cow<'_, str> = Cow::Owned("a".repeat(u16::MAX as usize + 1));
        let long_string_value = Amf0Value::from(long_string.clone());
        assert_eq!(long_string_value.as_string().unwrap(), &long_string);
        assert!(matches!(
            long_string_value.as_object(),
            Err(Amf0ReadError::WrongType {
                expected: Amf0Marker::Object,
                got: Amf0Marker::LongString
            })
        ));

        let object: Amf0Object = Cow::Owned(vec![(Cow::Borrowed("test"), Amf0Value::Number(1.0))]);
        let object_value = Amf0Value::from(object.clone());
        assert_eq!(object_value.as_object().unwrap(), &object);
        assert!(matches!(
            object_value.as_string(),
            Err(Amf0ReadError::WrongType {
                expected: Amf0Marker::String,
                got: Amf0Marker::Object
            })
        ));
    }
}
