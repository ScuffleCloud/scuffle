//! Video media specific boxes defined in ISO/IEC 14496-12 - 12.1

use scuffle_bytes_util::BytesCow;
use scuffle_bytes_util::zero_copy::{Deserialize, DeserializeSeed};

use super::SampleEntry;
use crate::{BoxHeader, FullBoxHeader, IsoBox};

/// Video media header
///
/// ISO/IEC 14496-12 - 12.1.2
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"vmhd", crate_path = crate)]
pub struct VideoMediaHeaderBox {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    pub graphicsmode: u16,
    pub opcolor: [u16; 3],
}

/// Visual sample entry
///
/// ISO/IEC 14496-12 - 12.1.3
///
/// Sub boxes:
/// - [`btrt`](super::BitRateBox)
/// - [`clap`](CleanApertureBox)
/// - [`pasp`](PixelAspectRatioBox)
/// - [`colr`](ColourInformationBox)
/// - [`clli`](ContentLightLevelBox)
/// - [`mdcv`](MasteringDisplayColourVolumeBox)
/// - [`cclv`](ContentColourVolumeBox)
/// - [`amve`](AmbientViewingEnvironmentBox)
/// - Any other boxes
#[derive(Debug)]
pub struct VisualSampleEntry {
    pub sample_entry: SampleEntry,
    pub pre_defined: u16,
    pub pre_defined2: [u32; 3],
    pub width: u16,
    pub height: u16,
    pub horiz_resolution: u32,
    pub vert_resolution: u32,
    pub frame_count: u16,
    pub compressor_name: [u8; 32],
    pub depth: u16,
    pub pre_defined4: i16,
}

impl<'a> Deserialize<'a> for VisualSampleEntry {
    fn deserialize<R>(mut reader: R) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let sample_entry = SampleEntry::deserialize(&mut reader)?;
        let pre_defined = u16::deserialize(&mut reader)?;
        u16::deserialize(&mut reader)?; // reserved
        let pre_defined2 = <[u32; 3]>::deserialize(&mut reader)?;
        let width = u16::deserialize(&mut reader)?;
        let height = u16::deserialize(&mut reader)?;
        let horiz_resolution = u32::deserialize(&mut reader)?;
        let vert_resolution = u32::deserialize(&mut reader)?;
        u32::deserialize(&mut reader)?; // reserved
        let frame_count = u16::deserialize(&mut reader)?;
        let compressor_name = <[u8; 32]>::deserialize(&mut reader)?;
        let depth = u16::deserialize(&mut reader)?;
        let pre_defined4 = i16::deserialize(&mut reader)?;

        Ok(Self {
            sample_entry,
            pre_defined,
            pre_defined2,
            width,
            height,
            horiz_resolution,
            vert_resolution,
            frame_count,
            compressor_name,
            depth,
            pre_defined4,
        })
    }
}

/// Clean aperture box
///
/// ISO/IEC 14496-12 - 12.1.4
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"clap", crate_path = crate)]
pub struct CleanApertureBox {
    #[iso_box(header)]
    pub header: BoxHeader,
    pub clean_aperture_width_n: u32,
    pub clean_aperture_width_d: u32,
    pub clean_aperture_height_n: u32,
    pub clean_aperture_height_d: u32,
    pub horiz_off_n: u32,
    pub horiz_off_d: u32,
    pub vert_off_n: u32,
    pub vert_off_d: u32,
}

/// Pixel aspect ratio box
///
/// ISO/IEC 14496-12 - 12.1.4
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"pasp", crate_path = crate)]
pub struct PixelAspectRatioBox {
    #[iso_box(header)]
    pub header: BoxHeader,
    pub h_spacing: u32,
    pub v_spacing: u32,
}

/// Colour information
///
/// ISO/IEC 14496-12 - 12.1.5
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"colr", crate_path = crate)]
pub struct ColourInformationBox<'a> {
    #[iso_box(header)]
    pub header: BoxHeader,
    pub colour_info: ColourInformation<'a>,
}

#[derive(Debug)]
pub enum ColourInformation<'a> {
    Nclx(NclxColourInformation),
    RIcc { icc_profile: BytesCow<'a> },
    Prof { icc_profile: BytesCow<'a> },
    Other(BytesCow<'a>),
}

impl<'a> Deserialize<'a> for ColourInformation<'a> {
    fn deserialize<R>(mut reader: R) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let colour_type = <[u8; 4]>::deserialize(&mut reader)?;

        match &colour_type {
            b"nclx" => {
                let colour_info = NclxColourInformation::deserialize(&mut reader)?;
                Ok(ColourInformation::Nclx(colour_info))
            }
            b"rICC" => {
                let icc_profile = reader.try_read_to_end()?;
                Ok(ColourInformation::RIcc { icc_profile })
            }
            b"prof" => {
                let icc_profile = reader.try_read_to_end()?;
                Ok(ColourInformation::Prof { icc_profile })
            }
            _ => Ok(Self::Other(reader.try_read_to_end()?)),
        }
    }
}

#[derive(Debug)]
pub struct NclxColourInformation {
    pub colour_primaries: u16,
    pub transfer_characteristics: u16,
    pub matrix_coefficients: u16,
    pub full_range_flag: bool,
}

impl<'a> Deserialize<'a> for NclxColourInformation {
    fn deserialize<R>(mut reader: R) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let colour_primaries = u16::deserialize(&mut reader)?;
        let transfer_characteristics = u16::deserialize(&mut reader)?;
        let matrix_coefficients = u16::deserialize(&mut reader)?;
        let full_range_flag = u8::deserialize(&mut reader)? >> 7 != 0;

        Ok(Self {
            colour_primaries,
            transfer_characteristics,
            matrix_coefficients,
            full_range_flag,
        })
    }
}

/// Content light level
///
/// ISO/IEC 144496-12 - 12.1.6
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"clli", crate_path = crate)]
pub struct ContentLightLevelBox {
    #[iso_box(header)]
    pub header: BoxHeader,
    pub max_content_light_level: u16,
    pub max_pic_average_light_level: u16,
}

/// Mastering display colour volume
///
/// ISO/IEC 144496-12 - 12.1.7
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"mdcv", crate_path = crate)]
pub struct MasteringDisplayColourVolumeBox {
    #[iso_box(header)]
    pub header: BoxHeader,
    pub display_primaries: [[u16; 2]; 6],
    pub white_point_x: u16,
    pub white_point_y: u16,
    pub max_display_mastering_luminance: u32,
    pub min_display_mastering_luminance: u32,
}

/// Content colour volume
///
/// ISO/IEC 144496-12 - 12.1.8
#[derive(Debug)]
pub struct ContentColourVolumeBox {
    pub header: BoxHeader,
    pub reserved1: bool,
    pub reserved2: bool,
    pub ccv_primaries_present_flag: bool,
    pub ccv_min_luminance_value_present_flag: bool,
    pub ccv_max_luminance_value_present_flag: bool,
    pub ccv_avg_luminance_value_present_flag: bool,
    pub ccv_reserved_zero_2bits: u8,
    pub ccv_primaries: Option<[[i32; 2]; 3]>,
    pub ccv_min_luminance_value: Option<u32>,
    pub ccv_max_luminance_value: Option<u32>,
    pub ccv_avg_luminance_value: Option<u32>,
}

impl IsoBox for ContentColourVolumeBox {
    type Header = BoxHeader;

    const TYPE: [u8; 4] = *b"cclv";
}

impl<'a> Deserialize<'a> for ContentColourVolumeBox {
    fn deserialize<R>(mut reader: R) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let header = BoxHeader::deserialize(&mut reader)?;
        Self::deserialize_seed(reader, header)
    }
}

impl<'a> DeserializeSeed<'a, BoxHeader> for ContentColourVolumeBox {
    fn deserialize_seed<R>(mut reader: R, seed: BoxHeader) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let byte = u8::deserialize(&mut reader)?;
        let reserved1 = ((byte >> 7) & 0b1) != 0;
        let reserved2 = ((byte >> 6) & 0b1) != 0;
        let ccv_primaries_present_flag = ((byte >> 5) & 0b1) != 0;
        let ccv_min_luminance_value_present_flag = ((byte >> 4) & 0b1) != 0;
        let ccv_max_luminance_value_present_flag = ((byte >> 3) & 0b1) != 0;
        let ccv_avg_luminance_value_present_flag = ((byte >> 2) & 0b1) != 0;
        let ccv_reserved_zero_2bits = byte & 0b11;

        let ccv_primaries = if ccv_primaries_present_flag {
            Some(<[[i32; 2]; 3]>::deserialize(&mut reader)?)
        } else {
            None
        };

        let ccv_min_luminance_value = if ccv_min_luminance_value_present_flag {
            Some(u32::deserialize(&mut reader)?)
        } else {
            None
        };

        let ccv_max_luminance_value = if ccv_max_luminance_value_present_flag {
            Some(u32::deserialize(&mut reader)?)
        } else {
            None
        };

        let ccv_avg_luminance_value = if ccv_avg_luminance_value_present_flag {
            Some(u32::deserialize(&mut reader)?)
        } else {
            None
        };

        Ok(Self {
            header: seed,
            reserved1,
            reserved2,
            ccv_primaries_present_flag,
            ccv_min_luminance_value_present_flag,
            ccv_max_luminance_value_present_flag,
            ccv_avg_luminance_value_present_flag,
            ccv_reserved_zero_2bits,
            ccv_primaries,
            ccv_min_luminance_value,
            ccv_max_luminance_value,
            ccv_avg_luminance_value,
        })
    }
}

/// Ambient viewing environment
///
/// ISO/IEC 144496-12 - 12.1.9
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"amve", crate_path = crate)]
pub struct AmbientViewingEnvironmentBox {
    #[iso_box(header)]
    pub header: BoxHeader,
    pub ambient_illuminance: u32,
    pub ambient_light_x: u16,
    pub ambient_light_y: u16,
}
