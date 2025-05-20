//! Video media specific boxes defined in ISO/IEC 14496-12 - 12.1

use scuffle_bytes_util::zero_copy::{Deserialize, DeserializeSeed, Serialize};
use scuffle_bytes_util::{BitWriter, BytesCow};

use super::SampleEntry;
use crate::{BoxHeader, FullBoxHeader, IsoBox, IsoSized};

/// Video media header
///
/// ISO/IEC 14496-12 - 12.1.2
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"vmhd", crate_path = crate)]
pub struct VideoMediaHeaderBox {
    pub full_header: FullBoxHeader,
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
    pub reserved1: u16,
    pub pre_defined2: [u32; 3],
    pub width: u16,
    pub height: u16,
    pub horiz_resolution: u32,
    pub vert_resolution: u32,
    pub reserved2: u32,
    pub frame_count: u16,
    pub compressor_name: [char; 32],
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
        let reserved1 = u16::deserialize(&mut reader)?;
        let pre_defined2 = <[u32; 3]>::deserialize(&mut reader)?;
        let width = u16::deserialize(&mut reader)?;
        let height = u16::deserialize(&mut reader)?;
        let horiz_resolution = u32::deserialize(&mut reader)?;
        let vert_resolution = u32::deserialize(&mut reader)?;
        let reserved2 = u32::deserialize(&mut reader)?;
        let frame_count = u16::deserialize(&mut reader)?;
        let compressor_name = <[char; 32]>::deserialize(&mut reader)?;
        let depth = u16::deserialize(&mut reader)?;
        let pre_defined4 = i16::deserialize(&mut reader)?;

        Ok(Self {
            sample_entry,
            pre_defined,
            reserved1,
            pre_defined2,
            width,
            height,
            horiz_resolution,
            vert_resolution,
            reserved2,
            frame_count,
            compressor_name,
            depth,
            pre_defined4,
        })
    }
}

impl Serialize for VisualSampleEntry {
    fn serialize<W>(&self, mut writer: W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        self.sample_entry.serialize(&mut writer)?;
        self.pre_defined.serialize(&mut writer)?;
        self.reserved1.serialize(&mut writer)?;
        self.pre_defined2.serialize(&mut writer)?;
        self.width.serialize(&mut writer)?;
        self.height.serialize(&mut writer)?;
        self.horiz_resolution.serialize(&mut writer)?;
        self.vert_resolution.serialize(&mut writer)?;
        self.reserved2.serialize(&mut writer)?;
        self.frame_count.serialize(&mut writer)?;
        self.compressor_name.serialize(&mut writer)?;
        self.depth.serialize(&mut writer)?;
        self.pre_defined4.serialize(&mut writer)?;
        Ok(())
    }
}

impl IsoSized for VisualSampleEntry {
    fn size(&self) -> usize {
        self.sample_entry.size()
            + 2 // pre_defined
            + 2 // reserved1
            + self.pre_defined2.size() // pre_defined2
            + 2 // width
            + 2 // height
            + 4 // horiz_resolution
            + 4 // vert_resolution
            + 4 // reserved2
            + 2 // frame_count
            + self.compressor_name.size() // compressor_name
            + 2 // depth
            + 2 // pre_defined4
    }
}

/// Clean aperture box
///
/// ISO/IEC 14496-12 - 12.1.4
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"clap", crate_path = crate)]
pub struct CleanApertureBox {
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
    pub h_spacing: u32,
    pub v_spacing: u32,
}

/// Colour information
///
/// ISO/IEC 14496-12 - 12.1.5
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"colr", crate_path = crate)]
pub struct ColourInformationBox<'a> {
    pub colour_info: ColourInformation<'a>,
}

#[derive(Debug)]
pub enum ColourInformation<'a> {
    Nclx(NclxColourInformation),
    RIcc { icc_profile: BytesCow<'a> },
    Prof { icc_profile: BytesCow<'a> },
    Other { colour_type: [u8; 4], data: BytesCow<'a> },
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
            _ => Ok(Self::Other {
                colour_type,
                data: reader.try_read_to_end()?,
            }),
        }
    }
}

impl Serialize for ColourInformation<'_> {
    fn serialize<W>(&self, mut writer: W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        match self {
            ColourInformation::Nclx(info) => {
                b"nclx".serialize(&mut writer)?;
                info.serialize(&mut writer)?;
            }
            ColourInformation::RIcc { icc_profile } => {
                b"rICC".serialize(&mut writer)?;
                icc_profile.serialize(&mut writer)?;
            }
            ColourInformation::Prof { icc_profile } => {
                b"prof".serialize(&mut writer)?;
                icc_profile.serialize(&mut writer)?;
            }
            ColourInformation::Other { colour_type, data } => {
                colour_type.serialize(&mut writer)?;
                data.serialize(&mut writer)?;
            }
        }
        Ok(())
    }
}

impl IsoSized for ColourInformation<'_> {
    fn size(&self) -> usize {
        match self {
            ColourInformation::Nclx(info) => 4 + info.size(),
            ColourInformation::RIcc { icc_profile } => 4 + icc_profile.size(),
            ColourInformation::Prof { icc_profile } => 4 + icc_profile.size(),
            ColourInformation::Other { data, .. } => 4 + data.size(),
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
        Ok(Self {
            colour_primaries: u16::deserialize(&mut reader)?,
            transfer_characteristics: u16::deserialize(&mut reader)?,
            matrix_coefficients: u16::deserialize(&mut reader)?,
            full_range_flag: u8::deserialize(&mut reader)? >> 7 != 0,
        })
    }
}

impl Serialize for NclxColourInformation {
    fn serialize<W>(&self, mut writer: W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        self.colour_primaries.serialize(&mut writer)?;
        self.transfer_characteristics.serialize(&mut writer)?;
        self.matrix_coefficients.serialize(&mut writer)?;
        ((self.full_range_flag as u8) << 7).serialize(&mut writer)?;
        Ok(())
    }
}

impl IsoSized for NclxColourInformation {
    fn size(&self) -> usize {
        2 + 2 + 2 + 1
    }
}

/// Content light level
///
/// ISO/IEC 144496-12 - 12.1.6
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"clli", crate_path = crate)]
pub struct ContentLightLevelBox {
    pub max_content_light_level: u16,
    pub max_pic_average_light_level: u16,
}

/// Mastering display colour volume
///
/// ISO/IEC 144496-12 - 12.1.7
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"mdcv", crate_path = crate)]
pub struct MasteringDisplayColourVolumeBox {
    pub display_primaries: [[u16; 2]; 6],
    pub white_point_x: u16,
    pub white_point_y: u16,
    pub max_display_mastering_luminance: u32,
    pub min_display_mastering_luminance: u32,
}

/// Content colour volume
///
/// ISO/IEC 144496-12 - 12.1.8
#[derive(Debug, IsoBox)]
#[iso_box(box_type = b"cclv", skip_impl(deserialize_seed, serialize, sized), crate_path = crate)]
pub struct ContentColourVolumeBox {
    pub reserved1: bool,
    pub reserved2: bool,
    pub ccv_reserved_zero_2bits: u8,
    pub ccv_primaries: Option<[[i32; 2]; 3]>,
    pub ccv_min_luminance_value: Option<u32>,
    pub ccv_max_luminance_value: Option<u32>,
    pub ccv_avg_luminance_value: Option<u32>,
}

impl<'a> DeserializeSeed<'a, BoxHeader> for ContentColourVolumeBox {
    fn deserialize_seed<R>(mut reader: R, _seed: BoxHeader) -> std::io::Result<Self>
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
            reserved1,
            reserved2,
            ccv_reserved_zero_2bits,
            ccv_primaries,
            ccv_min_luminance_value,
            ccv_max_luminance_value,
            ccv_avg_luminance_value,
        })
    }
}

impl Serialize for ContentColourVolumeBox {
    fn serialize<W>(&self, writer: W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        let mut bit_writer = BitWriter::new(writer);

        bit_writer.write_bit(self.reserved1)?;
        bit_writer.write_bit(self.reserved2)?;
        bit_writer.write_bit(self.ccv_primaries.is_some())?;
        bit_writer.write_bit(self.ccv_min_luminance_value.is_some())?;
        bit_writer.write_bit(self.ccv_max_luminance_value.is_some())?;
        bit_writer.write_bit(self.ccv_avg_luminance_value.is_some())?;
        bit_writer.write_bits(self.ccv_reserved_zero_2bits as u64, 2)?;

        if let Some(ccv_primaries) = &self.ccv_primaries {
            ccv_primaries.serialize(&mut bit_writer)?;
        }
        if let Some(ccv_min_luminance_value) = &self.ccv_min_luminance_value {
            ccv_min_luminance_value.serialize(&mut bit_writer)?;
        }
        if let Some(ccv_max_luminance_value) = &self.ccv_max_luminance_value {
            ccv_max_luminance_value.serialize(&mut bit_writer)?;
        }
        if let Some(ccv_avg_luminance_value) = &self.ccv_avg_luminance_value {
            ccv_avg_luminance_value.serialize(&mut bit_writer)?;
        }

        Ok(())
    }
}

impl IsoSized for ContentColourVolumeBox {
    fn size(&self) -> usize {
        let mut size = 0;
        size += 1; // flags
        if let Some(ccv_primaries) = self.ccv_primaries {
            size += ccv_primaries.size();
        }
        if let Some(ccv_min_luminance_value) = self.ccv_min_luminance_value {
            size += ccv_min_luminance_value.size();
        }
        if let Some(ccv_max_luminance_value) = self.ccv_max_luminance_value {
            size += ccv_max_luminance_value.size();
        }
        if let Some(ccv_avg_luminance_value) = self.ccv_avg_luminance_value {
            size += ccv_avg_luminance_value.size();
        }

        Self::add_header_size(size)
    }
}

/// Ambient viewing environment
///
/// ISO/IEC 144496-12 - 12.1.9
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"amve", crate_path = crate)]
pub struct AmbientViewingEnvironmentBox {
    pub ambient_illuminance: u32,
    pub ambient_light_x: u16,
    pub ambient_light_y: u16,
}
