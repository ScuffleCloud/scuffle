use std::io;

use scuffle_bytes_util::zero_copy::{Deserialize, DeserializeSeed, Serialize, ZeroCopyReader};
use scuffle_bytes_util::{BytesCow, IoResultExt};

use super::{OriginalFormatBox, SchemeInformationBox, SchemeTypeBox};
use crate::{BoxHeader, FullBoxHeader, IsoBox, UnknownBox, Utf8String};

/// Restricted scheme information box
///
/// ISO/IEC 14496-12 - 8.15.3
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"rinf", crate_path = crate)]
pub struct RestrictedSchemeInfoBox<'a> {
    /// The contained [`OriginalFormatBox`]. (mandatory)
    #[iso_box(nested_box)]
    pub original_format: OriginalFormatBox,
    /// The contained [`SchemeTypeBox`]. (mandatory)
    #[iso_box(nested_box)]
    pub scheme_type: SchemeTypeBox,
    /// The contained [`SchemeInformationBox`]. (optional)
    #[iso_box(nested_box(collect))]
    pub info: Option<SchemeInformationBox<'a>>,
    /// The contained [`CompatibleSchemeTypeBox`]es. (any quantity)
    #[iso_box(nested_box(collect))]
    pub csch: Vec<CompatibleSchemeTypeBox>,
}

/// Stereo video box
///
/// ISO/IEC 14496-12 - 8.15.4.2
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"stvi", skip_impl(deserialize_seed), crate_path = crate)]
pub struct StereoVideoBox<'a> {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// An integer. A zero value indicates that the content may only be displayed on
    /// stereoscopic displays. When (`single_view_allowed` & 1) is equal to 1, it is allowed to display the
    /// right view on a monoscopic single-view display. When (`single_view_allowed` & 2) is equal to 2, it is
    /// allowed to display the left view on a monoscopic single-view display.
    pub single_view_allowed: u8,
    /// An integer that indicates the stereo arrangement scheme used and the stereo
    /// indication type according to the used scheme.
    /// The following values for `stereo_scheme` are specified:
    ///
    /// - `1`: The frame packing scheme as specified by the Frame packing arrangement Supplemental
    ///   Enhancement Information message of ISO/IEC 14496-10:2014.
    /// - `2`: The stereo scheme as specified in ISO/IEC 23000-11 for both frame/service compatible and
    ///   2D/3D mixed services.
    /// - `3`: The arrangement type scheme as specified in ISO/IEC 13818-2:2013, Annex D
    ///   a value of `VideoFramePackingType` as defined in ISO/IEC 23091-2.
    ///
    /// Other values of stereo_scheme are reserved.
    pub stereo_scheme: u32,
    /// Indicates the number of bytes for the `stereo_indication_type` field.
    pub length: u32,
    /// Indicates the stereo arrangement type according to the used stereo indication
    /// scheme. The syntax and semantics of `stereo_indication_type` depend on the value of `stereo_scheme`.
    /// The syntax and semantics for stereo_indication_type for the following values of stereo_
    /// scheme are specified as follows:
    ///
    /// - `stereo_scheme` equal to 1: The value of length shall be 4 and `stereo_indication_type` shall
    ///   be `unsigned int(32)` which contains the `frame_packing_arrangement_type` value from
    ///   ISO/IEC 14496-10:2014, Table D.8 ('Definition of frame_packing_arrangement_type').
    /// - `stereo_scheme` equal to 2: The value of length shall be 4 and `stereo_indication_type` shall be
    ///   `unsigned int(32)` which contains the type value from ISO/IEC 13818-2:2013, Table D.1
    ///   ('Definition of arrangement_type').
    /// - `stereo_scheme` equal to 3: The value of length shall be 2 and `stereo_indication_type` shall
    ///   contain two syntax elements of `unsigned int(8)`. The first syntax element shall contain the
    ///   stereoscopic composition type from ISO/IEC 23000-11:2009, Table 4. The least significant
    ///   bit of the second syntax element shall contain the value of `is_left_first` as specified in
    ///   ISO/IEC 23000-11:2009, subclause 8.4.3, while the other bits are reserved and shall be set
    ///   to 0.
    /// - `stereo_scheme` equal to 4: The value of length shall be 2 and `stereo_indication_type` shall
    ///   contain two syntax elements of `unsigned int(8)`. The first syntax element shall contain a
    ///   `VideoFramePackingType` from ISO/IEC 23091-2. The least significant bit of the second syntax
    ///   element shall contain the value of `QuincunxSamplingFlag` as specified in ISO/IEC 23091-2,
    ///   while the other bits are reserved and shall be set to 0. `PackedContentInterpretationType`
    ///   specified in ISO/IEC 23091-2 is inferred to be equal to 1.
    /// - `stereo_scheme` equal to 5: The value of length shall be 3 and `stereo_indication_type` shall contain
    ///   three syntax elements of type `unsigned int(8)`. The first syntax element shall contain a
    ///   `VideoFramePackingType` from ISO/IEC 23091-2. The least significant bit of the second syntax
    ///   element shall contain the value of `QuincunxSamplingFlag` as specified in ISO/IEC 23091-2,
    ///   while the other bits are reserved and shall be set to 0. The third syntax element shall contain
    ///   the `PackedContentInterpretationType` from ISO/IEC 23091-2.
    pub stereo_indication_type: BytesCow<'a>,
    /// Any other contained boxes.
    #[iso_box(nested_box(collect_unknown))]
    pub any_box: Vec<UnknownBox<'a>>,
}

impl<'a> DeserializeSeed<'a, BoxHeader> for StereoVideoBox<'a> {
    fn deserialize_seed<R>(mut reader: R, _seed: BoxHeader) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let full_header = FullBoxHeader::deserialize(&mut reader)?;

        let single_view_allowed = u32::deserialize(&mut reader)?;
        let single_view_allowed = (single_view_allowed & 0b11) as u8;
        let stereo_scheme = u32::deserialize(&mut reader)?;

        let length = u32::deserialize(&mut reader)?;
        let stereo_indication_type = reader.try_read(length as usize)?;

        let mut any_box = Vec::new();
        loop {
            let Some(box_header) = BoxHeader::deserialize(&mut reader).eof_to_none()? else {
                break;
            };
            let Some(unknown_box) = UnknownBox::deserialize_seed(&mut reader, box_header).eof_to_none()? else {
                break;
            };
            any_box.push(unknown_box);
        }

        Ok(Self {
            full_header,
            single_view_allowed,
            stereo_scheme,
            length,
            stereo_indication_type,
            any_box,
        })
    }
}

/// Compatible scheme type box
///
/// ISO/IEC 14496-12 - 8.15.5
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"csch", skip_impl(deserialize_seed, serialize), crate_path = crate)]
pub struct CompatibleSchemeTypeBox {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// The code defining the protection or restriction scheme, normally expressed as a four character code.
    pub scheme_type: [u8; 4],
    /// The version of the scheme (used to create the content).
    pub scheme_version: u32,
    /// An absolute URI allowing for the option of directing the user to a web-page if they do not
    /// have the scheme installed on their system.
    pub scheme_uri: Option<Utf8String>,
}

impl<'a> DeserializeSeed<'a, BoxHeader> for CompatibleSchemeTypeBox {
    fn deserialize_seed<R>(mut reader: R, _seed: BoxHeader) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let full_header = FullBoxHeader::deserialize(&mut reader)?;

        let scheme_type = <[u8; 4]>::deserialize(&mut reader)?;
        let scheme_version = u32::deserialize(&mut reader)?;
        let scheme_uri = if (*full_header.flags & 0x000001) != 0 {
            Some(Utf8String::deserialize(&mut reader)?)
        } else {
            None
        };

        Ok(Self {
            full_header,
            scheme_type,
            scheme_version,
            scheme_uri,
        })
    }
}

impl Serialize for CompatibleSchemeTypeBox {
    fn serialize<W>(&self, mut writer: W) -> io::Result<()>
    where
        W: std::io::Write,
    {
        self.serialize_box_header(&mut writer)?;
        self.full_header.serialize(&mut writer)?;

        self.scheme_type.serialize(&mut writer)?;
        self.scheme_version.serialize(&mut writer)?;

        if (*self.full_header.flags & 0x000001) != 0 {
            self.scheme_uri
                .as_ref()
                .ok_or(io::Error::new(io::ErrorKind::InvalidData, "scheme_uri is required"))?
                .serialize(&mut writer)?;
        }

        Ok(())
    }
}
