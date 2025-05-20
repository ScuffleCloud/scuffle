use std::io;

use scuffle_bytes_util::zero_copy::{Deserialize, DeserializeSeed, Serialize};

use super::StereoVideoBox;
use crate::{BoxHeader, FullBoxHeader, IsoBox, UnknownBox, Utf8String};

/// Protection scheme information box
///
/// ISO/IEC 14496-12 - 8.12.2
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"sinf", crate_path = crate)]
pub struct ProtectionSchemeInfoBox<'a> {
    #[iso_box(nested_box)]
    pub orginal_format: OriginalFormatBox,
    #[iso_box(nested_box(collect))]
    pub scheme_type: Option<SchemeTypeBox>,
    #[iso_box(nested_box(collect))]
    pub info: Option<SchemeInformationBox<'a>>,
}

/// Original format box
///
/// ISO/IEC 14496-12 - 8.12.3
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"frma", crate_path = crate)]
pub struct OriginalFormatBox {
    pub data_format: [u8; 4],
}

/// Scheme type box
///
/// ISO/IEC 14496-12 - 8.12.6
#[derive(Debug, IsoBox)]
#[iso_box(box_type = b"schm", skip_impl(deserialize_seed, serialize), crate_path = crate)]
pub struct SchemeTypeBox {
    pub full_header: FullBoxHeader,
    pub scheme_type: [u8; 4],
    pub scheme_version: u32,
    pub scheme_uri: Option<Utf8String>,
}

impl<'a> DeserializeSeed<'a, BoxHeader> for SchemeTypeBox {
    fn deserialize_seed<R>(mut reader: R, _seed: BoxHeader) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
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

impl Serialize for SchemeTypeBox {
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

/// Scheme information box
///
/// ISO/IEC 14496-12 - 8.12.7
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"schi", crate_path = crate)]
pub struct SchemeInformationBox<'a> {
    #[iso_box(nested_box(collect))]
    pub stvi: Option<StereoVideoBox<'a>>,
    #[iso_box(nested_box(collect_unknown))]
    pub boxes: Vec<UnknownBox<'a>>,
}

/// Scheme information box
///
/// ISO/IEC 14496-12 - 8.12.7
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"scrb", crate_path = crate)]
pub struct ScrambleSchemeInfoBox<'a> {
    #[iso_box(nested_box)]
    pub scheme_type_box: SchemeTypeBox,
    #[iso_box(nested_box(collect))]
    pub info: Option<SchemeInformationBox<'a>>,
}
