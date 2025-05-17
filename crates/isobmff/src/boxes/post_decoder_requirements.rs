use std::io;

use scuffle_bytes_util::zero_copy::{Deserialize, DeserializeSeed, Serialize, ZeroCopyReader};
use scuffle_bytes_util::{BytesCow, IoResultExt};

use super::{OriginalFormatBox, SchemeInformationBox, SchemeTypeBox};
use crate::{BoxHeader, FullBoxHeader, IsoBox, UnknownBox, Utf8String};

/// Restricted scheme information box
///
/// ISO/IEC 14496-12 - 8.15.3
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"rinf", crate_path = crate)]
pub struct RestrictedSchemeInfoBox<'a> {
    #[iso_box(header)]
    pub header: BoxHeader,
    #[iso_box(nested_box)]
    pub original_format: OriginalFormatBox,
    #[iso_box(nested_box)]
    pub scheme_type: SchemeTypeBox,
    #[iso_box(nested_box(collect))]
    pub info: Option<SchemeInformationBox<'a>>,
    #[iso_box(nested_box(collect))]
    pub csch: Vec<CompatibleSchemeTypeBox>,
}

/// Stereo video box
///
/// ISO/IEC 14496-12 - 8.15.4.2
#[derive(Debug, IsoBox)]
#[iso_box(box_type = b"stvi", skip_impl(deserialize_seed), crate_path = crate)]
pub struct StereoVideoBox<'a> {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    pub single_view_allowed: u8,
    pub stereo_scheme: u32,
    pub length: u32,
    pub stereo_indication_type: BytesCow<'a>,
    #[iso_box(nested_box(collect_unknown))]
    pub any_box: Vec<UnknownBox<'a>>,
}

impl<'a> DeserializeSeed<'a, FullBoxHeader> for StereoVideoBox<'a> {
    fn deserialize_seed<R>(mut reader: R, seed: FullBoxHeader) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
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
            header: seed,
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
#[derive(Debug, IsoBox)]
#[iso_box(box_type = b"csch", skip_impl(deserialize_seed, serialize), crate_path = crate)]
pub struct CompatibleSchemeTypeBox {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    pub scheme_type: [u8; 4],
    pub scheme_version: u32,
    pub scheme_uri: Option<Utf8String>,
}

impl<'a> DeserializeSeed<'a, FullBoxHeader> for CompatibleSchemeTypeBox {
    fn deserialize_seed<R>(mut reader: R, seed: FullBoxHeader) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let scheme_type = <[u8; 4]>::deserialize(&mut reader)?;
        let scheme_version = u32::deserialize(&mut reader)?;
        let scheme_uri = if (*seed.flags & 0x000001) != 0 {
            Some(Utf8String::deserialize(&mut reader)?)
        } else {
            None
        };

        Ok(Self {
            header: seed,
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
        self.header.serialize(&mut writer)?;

        self.scheme_type.serialize(&mut writer)?;
        self.scheme_version.serialize(&mut writer)?;

        if (*self.header.flags & 0x000001) != 0 {
            self.scheme_uri
                .as_ref()
                .ok_or(io::Error::new(io::ErrorKind::InvalidData, "scheme_uri is required"))?
                .serialize(&mut writer)?;
        }

        Ok(())
    }
}
