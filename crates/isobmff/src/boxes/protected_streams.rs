use scuffle_bytes_util::zero_copy::{Deserialize, DeserializeSeed};

use crate::{BoxHeader, FullBoxHeader, IsoBox, UnknownBox, Utf8String};

/// Protection scheme information box
///
/// ISO/IEC 14496-12 - 8.12.2
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"sinf", crate_path = crate)]
pub struct ProtectionSchemeInfoBox<'a> {
    #[iso_box(header)]
    pub header: BoxHeader,
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
    #[iso_box(header)]
    pub header: BoxHeader,
    pub data_format: [u8; 4],
}

/// Scheme type box
///
/// ISO/IEC 14496-12 - 8.12.6
#[derive(Debug)]
pub struct SchemeTypeBox {
    pub header: FullBoxHeader,
    pub scheme_type: [u8; 4],
    pub scheme_version: u32,
    pub scheme_uri: Option<Utf8String>,
}

impl IsoBox for SchemeTypeBox {
    type Header = FullBoxHeader;

    const TYPE: [u8; 4] = *b"schm";
}

impl<'a> Deserialize<'a> for SchemeTypeBox {
    fn deserialize<R>(mut reader: R) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let header = BoxHeader::deserialize(&mut reader)?;
        let header = FullBoxHeader::deserialize_seed(&mut reader, header)?;
        Self::deserialize_seed(&mut reader, header)
    }
}

impl<'a> DeserializeSeed<'a, FullBoxHeader> for SchemeTypeBox {
    fn deserialize_seed<R>(mut reader: R, seed: FullBoxHeader) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
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

/// Scheme information box
///
/// ISO/IEC 14496-12 - 8.12.7
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"schi", crate_path = crate)]
pub struct SchemeInformationBox<'a> {
    #[iso_box(header)]
    pub header: BoxHeader,
    #[iso_box(nested_box(collect_unknown))]
    pub boxes: Vec<UnknownBox<'a>>,
}

/// Scheme information box
///
/// ISO/IEC 14496-12 - 8.12.7
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"scrb", crate_path = crate)]
pub struct ScrambleSchemeInfoBox<'a> {
    #[iso_box(header)]
    pub header: BoxHeader,
    #[iso_box(nested_box)]
    pub scheme_type_box: SchemeTypeBox,
    #[iso_box(nested_box(collect))]
    pub info: Option<SchemeInformationBox<'a>>,
}
