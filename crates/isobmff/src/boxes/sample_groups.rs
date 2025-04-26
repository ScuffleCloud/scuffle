use std::io;

use scuffle_bytes_util::zero_copy::{Deserialize, DeserializeSeed, Serialize, ZeroCopyReader};
use scuffle_bytes_util::{BytesCow, IoResultExt};

use crate::IsoSized;

/// All defined sample group description entries.
#[derive(Debug, PartialEq, Eq)]
pub enum SampleGroupDescriptionEntry<'a> {
    /// `roll`
    RollRecovery(RollRecoveryEntry),
    /// `prol`
    AudioPreRoll(AudioPreRollEntry),
    /// `rash`
    RateShare(RateShareEntry),
    /// `alst`
    AlternativeStartup(AlternativeStartupEntry),
    /// `rap `
    VisualRandomAccess(VisualRandomAccessEntry),
    /// `tele`
    TemporalLevel(TemporalLevelEntry),
    /// `sap `
    SAP(SAPEntry),
    /// `stmi`
    SampleToMetadataItem(SampleToMetadataItemEntry),
    /// `drap`
    VisualDRAP(VisualDRAPEntry),
    /// `pasr`
    PixelAspectRatio(PixelAspectRatioEntry),
    /// `casg`
    CleanAperture(CleanApertureEntry),
    /// Unknown entry
    Unknown {
        /// The grouping type of the entry
        grouping_type: [u8; 4],
        /// The data
        data: BytesCow<'a>,
    },
}

impl<'a> DeserializeSeed<'a, ([u8; 4], Option<u32>)> for SampleGroupDescriptionEntry<'a> {
    fn deserialize_seed<R>(mut reader: R, seed: ([u8; 4], Option<u32>)) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let (grouping_type, length) = seed;

        match &grouping_type {
            b"roll" => Ok(Self::RollRecovery(RollRecoveryEntry::deserialize(reader)?)),
            b"prol" => Ok(Self::AudioPreRoll(AudioPreRollEntry::deserialize(reader)?)),
            b"rash" => Ok(Self::RateShare(RateShareEntry::deserialize(reader)?)),
            b"alst" => Ok(Self::AlternativeStartup(AlternativeStartupEntry::deserialize(reader)?)),
            b"rap " => Ok(Self::VisualRandomAccess(VisualRandomAccessEntry::deserialize(reader)?)),
            b"tele" => Ok(Self::TemporalLevel(TemporalLevelEntry::deserialize(reader)?)),
            b"sap " => Ok(Self::SAP(SAPEntry::deserialize(reader)?)),
            b"stmi" => Ok(Self::SampleToMetadataItem(SampleToMetadataItemEntry::deserialize(reader)?)),
            b"drap" => Ok(Self::VisualDRAP(VisualDRAPEntry::deserialize(reader)?)),
            b"pasr" => Ok(Self::PixelAspectRatio(PixelAspectRatioEntry::deserialize(reader)?)),
            b"casg" => Ok(Self::CleanAperture(CleanApertureEntry::deserialize(reader)?)),
            _ => {
                let data = if let Some(length) = length {
                    reader.try_read(length as usize)?
                } else {
                    BytesCow::new()
                };
                Ok(Self::Unknown { grouping_type, data })
            }
        }
    }
}

impl Serialize for SampleGroupDescriptionEntry<'_> {
    fn serialize<W>(&self, mut writer: W) -> io::Result<()>
    where
        W: std::io::Write,
    {
        match self {
            Self::RollRecovery(entry) => entry.serialize(&mut writer),
            Self::AudioPreRoll(entry) => entry.serialize(&mut writer),
            Self::RateShare(entry) => entry.serialize(&mut writer),
            Self::AlternativeStartup(entry) => entry.serialize(&mut writer),
            Self::VisualRandomAccess(entry) => entry.serialize(&mut writer),
            Self::TemporalLevel(entry) => entry.serialize(&mut writer),
            Self::SAP(entry) => entry.serialize(&mut writer),
            Self::SampleToMetadataItem(entry) => entry.serialize(&mut writer),
            Self::VisualDRAP(entry) => entry.serialize(&mut writer),
            Self::PixelAspectRatio(entry) => entry.serialize(&mut writer),
            Self::CleanAperture(entry) => entry.serialize(&mut writer),
            Self::Unknown { data, .. } => data.serialize(&mut writer),
        }
    }
}

impl IsoSized for SampleGroupDescriptionEntry<'_> {
    fn size(&self) -> usize {
        match self {
            Self::RollRecovery(entry) => entry.size(),
            Self::AudioPreRoll(entry) => entry.size(),
            Self::RateShare(entry) => entry.size(),
            Self::AlternativeStartup(entry) => entry.size(),
            Self::VisualRandomAccess(entry) => entry.size(),
            Self::TemporalLevel(entry) => entry.size(),
            Self::SAP(entry) => entry.size(),
            Self::SampleToMetadataItem(entry) => entry.size(),
            Self::VisualDRAP(entry) => entry.size(),
            Self::PixelAspectRatio(entry) => entry.size(),
            Self::CleanAperture(entry) => entry.size(),
            Self::Unknown { data, .. } => data.len(),
        }
    }
}

/// `VisualRollRecoveryEntry` and `AudioRollRecoveryEntry`
///
/// `roll`
///
/// ISO/IEC 14496-12 - 10.1
#[derive(Debug, PartialEq, Eq)]
pub struct RollRecoveryEntry {
    roll_distance: i16,
}

impl<'a> Deserialize<'a> for RollRecoveryEntry {
    fn deserialize<R>(reader: R) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        Ok(Self {
            roll_distance: i16::deserialize(reader)?,
        })
    }
}

impl Serialize for RollRecoveryEntry {
    fn serialize<W>(&self, mut writer: W) -> io::Result<()>
    where
        W: std::io::Write,
    {
        self.roll_distance.serialize(&mut writer)?;
        Ok(())
    }
}

impl IsoSized for RollRecoveryEntry {
    fn size(&self) -> usize {
        2 // roll_distance
    }
}

/// `AudioPreRollEntry`
///
/// `prol`
///
/// ISO/IEC 14496-12 - 10.1
#[derive(Debug, PartialEq, Eq)]
pub struct AudioPreRollEntry {
    roll_distance: i16,
}

impl<'a> Deserialize<'a> for AudioPreRollEntry {
    fn deserialize<R>(reader: R) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        Ok(AudioPreRollEntry {
            roll_distance: i16::deserialize(reader)?,
        })
    }
}

impl Serialize for AudioPreRollEntry {
    fn serialize<W>(&self, mut writer: W) -> io::Result<()>
    where
        W: std::io::Write,
    {
        self.roll_distance.serialize(&mut writer)?;
        Ok(())
    }
}

impl IsoSized for AudioPreRollEntry {
    fn size(&self) -> usize {
        2 // roll_distance
    }
}

/// Rate share sample group entry
///
/// `rash`
///
/// ISO/IEC 14496-12 - 10.2
#[derive(Debug, PartialEq, Eq)]
pub struct RateShareEntry {
    operation_point_count: u16,
    operation_points: Vec<RateShareEntryOperationPoint>,
    maximum_bitrate: u32,
    minimum_bitrate: u32,
    discard_priority: u8,
}

impl<'a> Deserialize<'a> for RateShareEntry {
    fn deserialize<R>(mut reader: R) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let operation_point_count = u16::deserialize(&mut reader)?;
        let mut operation_points = Vec::with_capacity(operation_point_count as usize);

        if operation_point_count == 1 {
            let target_rate_share = u16::deserialize(&mut reader)?;
            operation_points.push(RateShareEntryOperationPoint {
                target_rate_share,
                available_bitrate: None,
            });
        } else {
            for _ in 0..operation_point_count {
                operation_points.push(RateShareEntryOperationPoint::deserialize(&mut reader)?);
            }
        }

        let maximum_bitrate = u32::deserialize(&mut reader)?;
        let minimum_bitrate = u32::deserialize(&mut reader)?;
        let discard_priority = u8::deserialize(&mut reader)?;

        Ok(Self {
            operation_point_count,
            operation_points,
            maximum_bitrate,
            minimum_bitrate,
            discard_priority,
        })
    }
}

impl Serialize for RateShareEntry {
    fn serialize<W>(&self, mut writer: W) -> io::Result<()>
    where
        W: std::io::Write,
    {
        self.operation_point_count.serialize(&mut writer)?;
        for operation_point in &self.operation_points {
            if self.operation_point_count > 1 && operation_point.available_bitrate.is_none() {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "available_bitrate is required"));
            }
            operation_point.serialize(&mut writer)?;
        }
        self.maximum_bitrate.serialize(&mut writer)?;
        self.minimum_bitrate.serialize(&mut writer)?;
        self.discard_priority.serialize(&mut writer)?;
        Ok(())
    }
}

impl IsoSized for RateShareEntry {
    fn size(&self) -> usize {
        2 + self.operation_points.size() + 4 + 4 + 1
    }
}

/// Operation point in [`RateShareEntry`].
#[derive(Debug, PartialEq, Eq)]
pub struct RateShareEntryOperationPoint {
    /// An integer. A non-zero value indicates the percentage of available bandwidth that
    /// should be allocated to the media for each operation point. The value of the first (last) operation
    /// point applies to lower (higher) available bitrates than the operation point itself. The target
    /// rate share between operation points is bounded by the target rate shares of the corresponding
    /// operation points. A zero value indicates that no information on the preferred rate share percentage
    /// is provided.
    pub target_rate_share: u16,
    /// A positive integer that defines an operation point (in kilobits per second). It is the
    /// total available bitrate that can be allocated in shares to tracks. Each entry shall be greater than the
    /// previous entry.
    pub available_bitrate: Option<u32>,
}

impl<'a> Deserialize<'a> for RateShareEntryOperationPoint {
    fn deserialize<R>(mut reader: R) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let available_bitrate = u32::deserialize(&mut reader)?;
        let target_rate_share = u16::deserialize(&mut reader)?;

        Ok(Self {
            available_bitrate: Some(available_bitrate),
            target_rate_share,
        })
    }
}

impl Serialize for RateShareEntryOperationPoint {
    fn serialize<W>(&self, mut writer: W) -> io::Result<()>
    where
        W: std::io::Write,
    {
        if let Some(available_bitrate) = &self.available_bitrate {
            available_bitrate.serialize(&mut writer)?;
        }
        self.target_rate_share.serialize(&mut writer)?;
        Ok(())
    }
}

impl IsoSized for RateShareEntryOperationPoint {
    fn size(&self) -> usize {
        if self.available_bitrate.is_some() {
            4 + 2 // available_bitrate + target_rate_share
        } else {
            2 // target_rate_share
        }
    }
}

/// Alternative startup sequences
///
/// `alst`
///
/// ISO/IEC 14496-12 - 10.3
#[derive(Debug, PartialEq, Eq)]
pub struct AlternativeStartupEntry {
    roll_count: u16,
    first_output_sample: u16,
    sample_offset: Vec<u32>,
    nums: Vec<AlternativeStartupEntryNums>,
}

impl<'a> Deserialize<'a> for AlternativeStartupEntry {
    fn deserialize<R>(mut reader: R) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let roll_count = u16::deserialize(&mut reader)?;
        let first_output_sample = u16::deserialize(&mut reader)?;

        let mut sample_offset = Vec::with_capacity(roll_count as usize);
        for _ in 0..roll_count {
            sample_offset.push(u32::deserialize(&mut reader)?);
        }

        let mut nums = Vec::new();
        loop {
            let Some(num) = AlternativeStartupEntryNums::deserialize(&mut reader).eof_to_none()? else {
                break;
            };
            nums.push(num);
        }

        Ok(Self {
            roll_count,
            first_output_sample,
            sample_offset,
            nums,
        })
    }
}

impl Serialize for AlternativeStartupEntry {
    fn serialize<W>(&self, mut writer: W) -> io::Result<()>
    where
        W: std::io::Write,
    {
        self.roll_count.serialize(&mut writer)?;
        self.first_output_sample.serialize(&mut writer)?;
        for offset in &self.sample_offset {
            offset.serialize(&mut writer)?;
        }
        for num in &self.nums {
            num.serialize(&mut writer)?;
        }
        Ok(())
    }
}

impl IsoSized for AlternativeStartupEntry {
    fn size(&self) -> usize {
        2 + 2 + self.sample_offset.size() + self.nums.size()
    }
}

/// Number of samples in an [`AlternativeStartupEntry`].
///
/// Indicates the sample output rate within the
/// alternative startup sequence. The alternative startup sequence is divided into k consecutive pieces,
/// where each piece has a constant sample output rate which is unequal to that of the adjacent pieces.
/// The first piece starts from the sample indicated by `first_output_sample`. `num_output_samples[j]`
/// indicates the number of the output samples of the j-th piece of the alternative startup sequence.
/// `num_total_samples[j]` indicates the total number of samples, including those that are not in the
/// alternative startup sequence, from the first sample in the j-th piece that is output to the earlier one
/// (in composition order) of the sample that ends the alternative startup sequence and the sample
/// that immediately precedes the first output sample of the (j+1)th piece.
#[derive(Debug, PartialEq, Eq)]
pub struct AlternativeStartupEntryNums {
    /// `num_output_samples[j]`
    pub num_output_samples: u16,
    /// `num_total_samples[j]`
    pub num_total_samples: u16,
}

impl<'a> Deserialize<'a> for AlternativeStartupEntryNums {
    fn deserialize<R>(mut reader: R) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        Ok(Self {
            num_output_samples: u16::deserialize(&mut reader)?,
            num_total_samples: u16::deserialize(&mut reader)?,
        })
    }
}

impl Serialize for AlternativeStartupEntryNums {
    fn serialize<W>(&self, mut writer: W) -> io::Result<()>
    where
        W: std::io::Write,
    {
        self.num_output_samples.serialize(&mut writer)?;
        self.num_total_samples.serialize(&mut writer)?;
        Ok(())
    }
}

impl IsoSized for AlternativeStartupEntryNums {
    fn size(&self) -> usize {
        2 + 2 // num_output_samples + num_total_samples
    }
}

/// Random access point (RAP) sample group
///
/// `rap `
///
/// ISO/IEC 14496-12 - 10.4
#[derive(Debug, PartialEq, Eq)]
pub struct VisualRandomAccessEntry {
    num_leading_samples_known: bool,
    num_leading_samples: u8,
}

impl<'a> Deserialize<'a> for VisualRandomAccessEntry {
    fn deserialize<R>(mut reader: R) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let byte = u8::deserialize(&mut reader)?;
        let num_leading_samples_known = (byte & 0b1000_0000) != 0;
        let num_leading_samples = byte & 0b0111_1111;

        Ok(Self {
            num_leading_samples_known,
            num_leading_samples,
        })
    }
}

impl Serialize for VisualRandomAccessEntry {
    fn serialize<W>(&self, mut writer: W) -> io::Result<()>
    where
        W: std::io::Write,
    {
        let mut byte = (self.num_leading_samples_known as u8) << 7;
        byte |= self.num_leading_samples;
        byte.serialize(&mut writer)?;
        Ok(())
    }
}

impl IsoSized for VisualRandomAccessEntry {
    fn size(&self) -> usize {
        1 // num_leading_samples_known + num_leading_samples
    }
}

/// Temporal level sample group
///
/// `tele`
///
/// ISO/IEC 14496-12 - 10.5
#[derive(Debug, PartialEq, Eq)]
pub struct TemporalLevelEntry {
    level_independently_decodable: bool,
}

impl<'a> Deserialize<'a> for TemporalLevelEntry {
    fn deserialize<R>(mut reader: R) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let level_independently_decodable = (u8::deserialize(&mut reader)? & 0b1000_0000) != 0;

        Ok(Self {
            level_independently_decodable,
        })
    }
}

impl Serialize for TemporalLevelEntry {
    fn serialize<W>(&self, mut writer: W) -> io::Result<()>
    where
        W: std::io::Write,
    {
        ((self.level_independently_decodable as u8) << 7).serialize(&mut writer)?;
        Ok(())
    }
}

impl IsoSized for TemporalLevelEntry {
    fn size(&self) -> usize {
        1 // level_independently_decodable
    }
}

/// Stream access point sample group
///
/// `sap `
///
/// ISO/IEC 14496-12 - 10.6
#[derive(Debug, PartialEq, Eq)]
pub struct SAPEntry {
    dependent_flag: bool,
    sap_type: u8,
}

impl<'a> Deserialize<'a> for SAPEntry {
    fn deserialize<R>(mut reader: R) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        // x000 xxxx
        let byte = u8::deserialize(&mut reader)?;
        let dependent_flag = (byte & 0b1000_0000) != 0;
        let sap_type = byte & 0b0000_1111;

        Ok(Self {
            dependent_flag,
            sap_type,
        })
    }
}

impl Serialize for SAPEntry {
    fn serialize<W>(&self, mut writer: W) -> io::Result<()>
    where
        W: std::io::Write,
    {
        let mut byte = (self.dependent_flag as u8) << 7;
        byte |= self.sap_type & 0b0000_1111;
        byte.serialize(&mut writer)?;
        Ok(())
    }
}

impl IsoSized for SAPEntry {
    fn size(&self) -> usize {
        1 // dependent_flag + sap_type
    }
}

/// Sample-to-item sample group
///
/// `stmi`
///
/// ISO/IEC 14496-12 - 10.7
#[derive(Debug, PartialEq, Eq)]
pub struct SampleToMetadataItemEntry {
    meta_box_handler_type: u32,
    num_items: u32,
    item_id: Vec<u32>,
}

impl<'a> Deserialize<'a> for SampleToMetadataItemEntry {
    fn deserialize<R>(mut reader: R) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let meta_box_handler_type = u32::deserialize(&mut reader)?;
        let num_items = u32::deserialize(&mut reader)?;

        let mut item_id = Vec::with_capacity(num_items as usize);
        for _ in 0..num_items {
            item_id.push(u32::deserialize(&mut reader)?);
        }

        Ok(Self {
            meta_box_handler_type,
            num_items,
            item_id,
        })
    }
}

impl Serialize for SampleToMetadataItemEntry {
    fn serialize<W>(&self, mut writer: W) -> io::Result<()>
    where
        W: std::io::Write,
    {
        self.meta_box_handler_type.serialize(&mut writer)?;
        self.num_items.serialize(&mut writer)?;
        for id in &self.item_id {
            id.serialize(&mut writer)?;
        }
        Ok(())
    }
}

impl IsoSized for SampleToMetadataItemEntry {
    fn size(&self) -> usize {
        4 + 4 + self.item_id.size() // meta_box_handler_type + num_items + item_id
    }
}

/// Dependent random access point (DRAP) sample group
///
/// `drap`
///
/// ISO/IEC 14496-12 - 10.8
#[derive(Debug, PartialEq, Eq)]
pub struct VisualDRAPEntry {
    drap_type: u8,
}

impl<'a> Deserialize<'a> for VisualDRAPEntry {
    fn deserialize<R>(mut reader: R) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let drap_type = ((u32::deserialize(&mut reader)? >> 29) & 0b111) as u8;
        Ok(Self { drap_type })
    }
}

impl Serialize for VisualDRAPEntry {
    fn serialize<W>(&self, mut writer: W) -> io::Result<()>
    where
        W: std::io::Write,
    {
        let byte = ((self.drap_type & 0b1111) as u32) << 29;
        byte.serialize(&mut writer)?;
        Ok(())
    }
}

impl IsoSized for VisualDRAPEntry {
    fn size(&self) -> usize {
        4 // drap_type
    }
}

/// Pixel Aspect Ratio Sample Grouping
///
/// `pasr`
///
/// ISO/IEC 14496-12 - 10.9
#[derive(Debug, PartialEq, Eq)]
pub struct PixelAspectRatioEntry {
    h_spacing: u32,
    v_spacing: u32,
}

impl<'a> Deserialize<'a> for PixelAspectRatioEntry {
    fn deserialize<R>(mut reader: R) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        Ok(Self {
            h_spacing: u32::deserialize(&mut reader)?,
            v_spacing: u32::deserialize(&mut reader)?,
        })
    }
}

impl Serialize for PixelAspectRatioEntry {
    fn serialize<W>(&self, mut writer: W) -> io::Result<()>
    where
        W: std::io::Write,
    {
        self.h_spacing.serialize(&mut writer)?;
        self.v_spacing.serialize(&mut writer)?;
        Ok(())
    }
}

impl IsoSized for PixelAspectRatioEntry {
    fn size(&self) -> usize {
        4 + 4 // h_spacing + v_spacing
    }
}

/// Clean Aperture Sample Grouping
///
/// `casg`
///
/// ISO/IEC 14496-12 - 10.10
#[derive(Debug, PartialEq, Eq)]
pub struct CleanApertureEntry {
    clean_aperture_width_n: u32,
    clean_aperture_width_d: u32,
    clean_aperture_height_n: u32,
    clean_aperture_height_d: u32,
    horiz_off_n: u32,
    horiz_off_d: u32,
    vert_off_n: u32,
    vert_off_d: u32,
}

impl<'a> Deserialize<'a> for CleanApertureEntry {
    fn deserialize<R>(mut reader: R) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        Ok(Self {
            clean_aperture_width_n: u32::deserialize(&mut reader)?,
            clean_aperture_width_d: u32::deserialize(&mut reader)?,
            clean_aperture_height_n: u32::deserialize(&mut reader)?,
            clean_aperture_height_d: u32::deserialize(&mut reader)?,
            horiz_off_n: u32::deserialize(&mut reader)?,
            horiz_off_d: u32::deserialize(&mut reader)?,
            vert_off_n: u32::deserialize(&mut reader)?,
            vert_off_d: u32::deserialize(&mut reader)?,
        })
    }
}

impl Serialize for CleanApertureEntry {
    fn serialize<W>(&self, mut writer: W) -> io::Result<()>
    where
        W: std::io::Write,
    {
        self.clean_aperture_width_n.serialize(&mut writer)?;
        self.clean_aperture_width_d.serialize(&mut writer)?;
        self.clean_aperture_height_n.serialize(&mut writer)?;
        self.clean_aperture_height_d.serialize(&mut writer)?;
        self.horiz_off_n.serialize(&mut writer)?;
        self.horiz_off_d.serialize(&mut writer)?;
        self.vert_off_n.serialize(&mut writer)?;
        self.vert_off_d.serialize(&mut writer)?;
        Ok(())
    }
}

impl IsoSized for CleanApertureEntry {
    fn size(&self) -> usize {
        4 + 4 + 4 + 4 + 4 + 4 + 4 + 4 // clean_aperture_width_n + clean_aperture_width_d + clean_aperture_height_n + clean_aperture_height_d + horiz_off_n + horiz_off_d + vert_off_n + vert_off_d
    }
}
