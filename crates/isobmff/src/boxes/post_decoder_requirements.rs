use std::io;

use scuffle_bytes_util::IoResultExt;
use scuffle_bytes_util::zero_copy::{Deserialize, DeserializeSeed, ZeroCopyReader};

use super::{OriginalFormatBox, SchemeInformationBox, SchemeTypeBox};
use crate::{BoxHeader, BoxType, FullBoxHeader, IsoBox, UnknownBox, Utf8String};

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
#[derive(Debug)]
pub struct StereoVideoBox<'a> {
    pub header: FullBoxHeader,
    pub single_view_allowed: u8,
    pub stereo_scheme: u32,
    pub length: u32,
    pub stereo_indication_type: Vec<u8>,
    pub any_box: Vec<UnknownBox<'a>>,
}

impl IsoBox for StereoVideoBox<'_> {
    type Header = FullBoxHeader;

    const TYPE: BoxType = BoxType::FourCc(*b"stvi");
}

impl<'a> Deserialize<'a> for StereoVideoBox<'a> {
    fn deserialize<R>(mut reader: R) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let header = BoxHeader::deserialize(&mut reader)?;
        let header = FullBoxHeader::deserialize_seed(&mut reader, header)?;
        Self::deserialize_seed(reader, header)
    }
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
        let mut stereo_indication_type = Vec::with_capacity(length as usize);
        for _ in 0..length {
            stereo_indication_type.push(u8::deserialize(&mut reader)?);
        }

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
#[derive(Debug)]
pub struct CompatibleSchemeTypeBox {
    pub header: FullBoxHeader,
    pub scheme_type: [u8; 4],
    pub scheme_version: u32,
    pub scheme_uri: Option<Utf8String>,
}

impl IsoBox for CompatibleSchemeTypeBox {
    type Header = FullBoxHeader;

    const TYPE: BoxType = BoxType::FourCc(*b"csch");
}

impl<'a> Deserialize<'a> for CompatibleSchemeTypeBox {
    fn deserialize<R>(mut reader: R) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let header = BoxHeader::deserialize(&mut reader)?;
        let header = FullBoxHeader::deserialize_seed(&mut reader, header)?;
        Self::deserialize_seed(reader, header)
    }
}

impl<'a> DeserializeSeed<'a, FullBoxHeader> for CompatibleSchemeTypeBox {
    fn deserialize_seed<R>(mut reader: R, seed: FullBoxHeader) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let scheme_type = <[u8; 4]>::deserialize(&mut reader)?;
        let scheme_version = u32::deserialize(&mut reader)?;
        let scheme_uri = if seed.flags & 0x000001 != 0 {
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
