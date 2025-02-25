use std::borrow::Cow;

use num_derive::FromPrimitive;

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
    Object(Cow<'a, [(Cow<'a, str>, Amf0Value<'a>)]>),
    /// Null Type defined section 2.7
    Null,
    /// Undefined Type defined section 2.8
    ObjectEnd,
    /// LongString Type defined section 2.14
    LongString(Cow<'a, str>),
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
}
