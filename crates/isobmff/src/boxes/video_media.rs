//! Video media specific boxes defined in ISO/IEC 14496-12 - 12.1

use scuffle_bytes_util::BytesCow;
use scuffle_bytes_util::zero_copy::Deserialize;

use super::SampleEntry;
use crate::{BoxHeader, IsoBox};

/// Clean aperture box
///
/// ISO/IEC 14496-12 - 12.1.4
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"clap", crate_path = "crate")]
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
#[iso_box(box_type = b"pasp", crate_path = "crate")]
pub struct PixelAspectRatioBox {
    #[iso_box(header)]
    pub header: BoxHeader,
    pub h_spacing: u32,
    pub v_spacing: u32,
}

#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"colr", crate_path = "crate")]
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
