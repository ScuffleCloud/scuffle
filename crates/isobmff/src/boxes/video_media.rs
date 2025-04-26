//! Video media specific boxes defined in ISO/IEC 14496-12 - 12.1

use fixed::FixedU32;
use fixed::types::extra::U16;
use scuffle_bytes_util::zero_copy::{Deserialize, DeserializeSeed, Serialize};
use scuffle_bytes_util::{BitWriter, BytesCow};

use super::SampleEntry;
use crate::{BoxHeader, FullBoxHeader, IsoBox, IsoSized};

/// Video media header
///
/// ISO/IEC 14496-12 - 12.1.2
#[derive(IsoBox, Debug, PartialEq, Eq, Default)]
#[iso_box(box_type = b"vmhd", crate_path = crate)]
pub struct VideoMediaHeaderBox {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// A composition mode for this video track, from the following enumerated set,
    /// which may be extended by derived specifications:
    ///
    /// - copy = 0 copy over the existing image
    pub graphicsmode: u16,
    /// A set of 3 colour values (red, green, blue) available for use by graphics modes.
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
#[derive(Debug, PartialEq, Eq)]
pub struct VisualSampleEntry {
    /// The sample entry that this box inherits from.
    pub sample_entry: SampleEntry,
    /// Pre-defined 16 bits, must be 0.
    pub pre_defined: u16,
    /// Reserved 16 bits, must be 0.
    pub reserved1: u16,
    /// Pre-defined 3 * 32 bits, must be 0.
    pub pre_defined2: [u32; 3],
    /// The maximum visual `width` and `height` of the stream described by this sample description, in pixels.
    pub width: u16,
    /// See [`width`](Self::width).
    pub height: u16,
    /// Must be set to `0x00480000` (72 dpi).
    pub horiz_resolution: FixedU32<U16>,
    /// Must be set to `0x00480000` (72 dpi).
    pub vert_resolution: FixedU32<U16>,
    /// Reserved 32 bits, must be 0.
    pub reserved2: u32,
    /// How many frames of compressed video are stored in each sample. The default is
    /// 1, for one frame per sample; it may be more than 1 for multiple frames per sample.
    pub frame_count: u16,
    /// A name, for informative purposes. It is formatted in a fixed 32-byte field, with the
    /// first byte set to the number of bytes to be displayed, followed by that number of bytes of displayable
    /// data encoded using UTF-8, and then padding to complete 32 bytes total (including the size byte).
    /// The field may be set to 0.
    pub compressor_name: [u8; 32],
    /// One of the following values:
    ///
    /// - `0x0018`: images are in colour with no alpha.
    pub depth: u16,
    /// Pre-defined 16 bits, must be -1.
    pub pre_defined4: i16,
}

impl VisualSampleEntry {
    /// Creates a new [`VisualSampleEntry`] with the given parameters.
    pub fn new(sample_entry: SampleEntry, width: u16, height: u16, compressor_name: [u8; 32]) -> Self {
        Self {
            sample_entry,
            pre_defined: 0,
            reserved1: 0,
            pre_defined2: [0; 3],
            width,
            height,
            horiz_resolution: FixedU32::from_bits(0x00480000),
            vert_resolution: FixedU32::from_bits(0x00480000),
            reserved2: 0,
            frame_count: 1,
            compressor_name,
            depth: 0x0018,
            pre_defined4: -1,
        }
    }
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
        let horiz_resolution = FixedU32::from_bits(u32::deserialize(&mut reader)?);
        let vert_resolution = FixedU32::from_bits(u32::deserialize(&mut reader)?);
        let reserved2 = u32::deserialize(&mut reader)?;
        let frame_count = u16::deserialize(&mut reader)?;
        let compressor_name = <[u8; 32]>::deserialize(&mut reader)?;
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
        self.horiz_resolution.to_bits().serialize(&mut writer)?;
        self.vert_resolution.to_bits().serialize(&mut writer)?;
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
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"clap", crate_path = crate)]
pub struct CleanApertureBox {
    /// A fractional number which defines the width of the clean aperture image.
    pub clean_aperture_width_n: u32,
    /// A fractional number which defines the width of the clean aperture image.
    pub clean_aperture_width_d: u32,
    /// A fractional number which defines the height of the clean aperture image.
    pub clean_aperture_height_n: u32,
    /// A fractional number which defines the height of the clean aperture image.
    pub clean_aperture_height_d: u32,
    /// A fractional number which defines the horizontal offset between the clean
    /// aperture image centre and the full aperture image centre. Typically 0.
    pub horiz_off_n: u32,
    /// A fractional number which defines the horizontal offset between the clean
    /// aperture image centre and the full aperture image centre. Typically 0.
    pub horiz_off_d: u32,
    /// A fractional number which defines the vertical offset between clean aperture image
    /// centre and the full aperture image centre. Typically 0.
    pub vert_off_n: u32,
    /// A fractional number which defines the vertical offset between clean aperture image
    /// centre and the full aperture image centre. Typically 0.
    pub vert_off_d: u32,
}

/// Pixel aspect ratio box
///
/// ISO/IEC 14496-12 - 12.1.4
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"pasp", crate_path = crate)]
pub struct PixelAspectRatioBox {
    /// Define the relative width and height of a pixel.
    pub h_spacing: u32,
    /// Define the relative width and height of a pixel.
    pub v_spacing: u32,
}

impl Default for PixelAspectRatioBox {
    fn default() -> Self {
        Self {
            h_spacing: 1,
            v_spacing: 1,
        }
    }
}

/// Colour information
///
/// ISO/IEC 14496-12 - 12.1.5
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"colr", crate_path = crate)]
pub struct ColourInformationBox<'a> {
    /// The colour information.
    pub colour_info: ColourInformation<'a>,
}

/// Colour information in the [`ColourInformationBox`] box.
#[derive(Debug, PartialEq, Eq)]
pub enum ColourInformation<'a> {
    /// On-screen colour.
    Nclx(NclxColourInformation),
    /// Restricted ICC profile.
    RIcc {
        /// An ICC profile as defined in ISO 15076-1 or ICC.1 is supplied.
        icc_profile: BytesCow<'a>,
    },
    /// Restricted ICC profile.
    Prof {
        /// An ICC profile as defined in ISO 15076-1 or ICC.1 is supplied.
        icc_profile: BytesCow<'a>,
    },
    /// Other colour information.
    Other {
        /// The colour type.
        colour_type: [u8; 4],
        /// The colour info data.
        data: BytesCow<'a>,
    },
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

/// NCLX colour information in the [`ColourInformationBox`].
#[derive(Debug, PartialEq, Eq)]
pub struct NclxColourInformation {
    /// Carries a `ColourPrimaries` value as defined in ISO/IEC 23091-2.
    pub colour_primaries: u16,
    /// Carries a `TransferCharacteristics` value as defined in ISO/IEC 23091-2.
    pub transfer_characteristics: u16,
    /// Carries a `MatrixCoefficients` value as defined in ISO/IEC 23091-2.
    pub matrix_coefficients: u16,
    /// Carries a `VideoFullRangeFlag` as defined in ISO/IEC 23091-2.
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
///
/// It is functionally equivalent to, and shall be as described in, the Content light level
/// information SEI message in ITU-T H.265 | ISO/IEC 23008-2, with the addition that the provisions of
/// CTA-861-G, in which zero in some cases codes an unknown value, may be used.
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"clli", crate_path = crate)]
pub struct ContentLightLevelBox {
    /// See [`ContentLightLevelBox`].
    pub max_content_light_level: u16,
    /// See [`ContentLightLevelBox`].
    pub max_pic_average_light_level: u16,
}

/// Mastering display colour volume
///
/// ISO/IEC 144496-12 - 12.1.7
///
/// It is functionally equivalent to, and shall be as described in, the mastering display colour volume SEI message in
/// ITU-T H.265 | ISO/IEC 23008-2, with the addition that the provisions of CTA-861-G in which zero in
/// some cases codes an unknown value may be used.
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"mdcv", crate_path = crate)]
pub struct MasteringDisplayColourVolumeBox {
    /// See [`MasteringDisplayColourVolumeBox`].
    pub display_primaries: [[u16; 2]; 6],
    /// See [`MasteringDisplayColourVolumeBox`].
    pub white_point_x: u16,
    /// See [`MasteringDisplayColourVolumeBox`].
    pub white_point_y: u16,
    /// See [`MasteringDisplayColourVolumeBox`].
    pub max_display_mastering_luminance: u32,
    /// See [`MasteringDisplayColourVolumeBox`].
    pub min_display_mastering_luminance: u32,
}

/// Content colour volume
///
/// ISO/IEC 144496-12 - 12.1.8
///
/// It is functionally equivalent to, and shall be as described in, the content colour volume SEI message
/// in Rec. ITU-T H.265 | ISO/IEC 23008-2 except that the box, in a sample entry, applies to the associated
/// content and hence the initial two bits (corresponding to the `ccv_cancel_flag` and `ccv_persistence_flag`)
/// take the value 0.
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"cclv", skip_impl(deserialize_seed, serialize, sized), crate_path = crate)]
pub struct ContentColourVolumeBox {
    /// Reserved 1 bit, must be 0.
    pub reserved1: bool,
    /// Reserved 1 bit, must be 0.
    pub reserved2: bool,
    /// See [`ContentColourVolumeBox`].
    pub ccv_reserved_zero_2bits: u8,
    /// See [`ContentColourVolumeBox`].
    pub ccv_primaries: Option<[[i32; 2]; 3]>,
    /// See [`ContentColourVolumeBox`].
    pub ccv_min_luminance_value: Option<u32>,
    /// See [`ContentColourVolumeBox`].
    pub ccv_max_luminance_value: Option<u32>,
    /// See [`ContentColourVolumeBox`].
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
///
/// It is functionally equivalent to, and shall be as described in, the ambient viewing environment SEI message
/// in ITU-T H.265 |I ISO/IEC 23008-2.
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"amve", crate_path = crate)]
pub struct AmbientViewingEnvironmentBox {
    /// See [`AmbientViewingEnvironmentBox`].
    pub ambient_illuminance: u32,
    /// See [`AmbientViewingEnvironmentBox`].
    pub ambient_light_x: u16,
    /// See [`AmbientViewingEnvironmentBox`].
    pub ambient_light_y: u16,
}
