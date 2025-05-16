use scuffle_bytes_util::BitWriter;
use scuffle_bytes_util::zero_copy::{Deserialize, DeserializeSeed, Serialize, U24Be};

use super::Brand;
use crate::{BoxHeader, FullBoxHeader, IsoBox};

/// Segment type box
///
/// ISO/IEC 14496-12 - 8.16.2
#[derive(Debug, IsoBox)]
#[iso_box(box_type = b"styp", crate_path = crate)]
pub struct SegmentTypeBox {
    #[iso_box(header)]
    pub header: BoxHeader,
    #[iso_box(from = "[u8; 4]")]
    pub major_brand: Brand,
    pub minor_version: u32,
    #[iso_box(repeated, from = "[u8; 4]")]
    pub compatible_brands: Vec<Brand>,
}

/// Segment index box
///
/// ISO/IEC 14496-12 - 8.16.3
#[derive(Debug, IsoBox)]
#[iso_box(box_type = b"sidx", skip_impl(deserialize_seed, serialize), crate_path = crate)]
pub struct SegmentIndexBox {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    pub reference_id: u32,
    pub timescale: u32,
    pub earliest_presentation_time: u64,
    pub first_offset: u64,
    pub reserved: u16,
    pub reference_count: u16,
    pub references: Vec<SegmentIndexBoxReference>,
}

impl<'a> DeserializeSeed<'a, FullBoxHeader> for SegmentIndexBox {
    fn deserialize_seed<R>(mut reader: R, seed: FullBoxHeader) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let reference_id = u32::deserialize(&mut reader)?;
        let timescale = u32::deserialize(&mut reader)?;

        let earliest_presentation_time = if seed.version == 0 {
            u32::deserialize(&mut reader)? as u64
        } else {
            u64::deserialize(&mut reader)?
        };
        let first_offset = if seed.version == 0 {
            u32::deserialize(&mut reader)? as u64
        } else {
            u64::deserialize(&mut reader)?
        };

        let reserved = u16::deserialize(&mut reader)?;
        let reference_count = u16::deserialize(&mut reader)?;

        let mut references = Vec::with_capacity(reference_count as usize);
        for _ in 0..reference_count {
            references.push(SegmentIndexBoxReference::deserialize(&mut reader)?);
        }

        Ok(SegmentIndexBox {
            header: seed,
            reference_id,
            timescale,
            earliest_presentation_time,
            first_offset,
            reserved,
            reference_count,
            references,
        })
    }
}

impl Serialize for SegmentIndexBox {
    fn serialize<W>(&self, mut writer: W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        self.header.serialize(&mut writer)?;

        self.reference_id.serialize(&mut writer)?;
        self.timescale.serialize(&mut writer)?;

        if self.header.version == 0 {
            (self.earliest_presentation_time as u32).serialize(&mut writer)?;
            (self.first_offset as u32).serialize(&mut writer)?;
        } else {
            self.earliest_presentation_time.serialize(&mut writer)?;
            self.first_offset.serialize(&mut writer)?;
        }

        self.reserved.serialize(&mut writer)?;
        self.reference_count.serialize(&mut writer)?;

        for reference in &self.references {
            reference.serialize(&mut writer)?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct SegmentIndexBoxReference {
    pub reference_type: bool,
    pub referenced_size: u32,
    pub subsegment_duration: u32,
    pub starts_with_sap: bool,
    pub sap_type: u8,
    pub sap_delta_time: u32,
}

impl<'a> Deserialize<'a> for SegmentIndexBoxReference {
    fn deserialize<R>(mut reader: R) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let first_u32 = u32::deserialize(&mut reader)?;
        let reference_type = (first_u32 >> 31) != 0;
        let referenced_size = first_u32 & 0x7F_FF_FF_FF;

        let subsegment_duration = u32::deserialize(&mut reader)?;

        let third_u32 = u32::deserialize(&mut reader)?;
        let starts_with_sap = (third_u32 >> 31) != 0;
        let sap_type = ((third_u32 >> 28) & 0b111) as u8;
        let sap_delta_time = third_u32 & 0x0F_FF_FF_FF;

        Ok(SegmentIndexBoxReference {
            reference_type,
            referenced_size,
            subsegment_duration,
            starts_with_sap,
            sap_type,
            sap_delta_time,
        })
    }
}

impl Serialize for SegmentIndexBoxReference {
    fn serialize<W>(&self, writer: W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        let mut bit_writer = BitWriter::new(writer);

        bit_writer.write_bit(self.reference_type)?;
        bit_writer.write_bits(self.referenced_size as u64, 31)?;

        self.subsegment_duration.serialize(&mut bit_writer)?;

        bit_writer.write_bit(self.starts_with_sap)?;
        bit_writer.write_bits(self.sap_type as u64, 3)?;
        bit_writer.write_bits(self.sap_delta_time as u64, 28)?;

        Ok(())
    }
}

/// Subsegment index box
///
/// ISO/IEC 14496-12 - 8.16.4
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"ssix", crate_path = crate)]
pub struct SubsegmentIndexBox {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    pub subsegment_count: u32,
    #[iso_box(repeated)]
    pub subsegments: Vec<SubsegmentIndexBoxSubsegment>,
}

#[derive(Debug)]
pub struct SubsegmentIndexBoxSubsegment {
    pub range_count: u32,
    pub ranges: Vec<SubsegmentIndexBoxSubsegmentRange>,
}

impl<'a> Deserialize<'a> for SubsegmentIndexBoxSubsegment {
    fn deserialize<R>(mut reader: R) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let range_count = u32::deserialize(&mut reader)?;
        let mut ranges = Vec::with_capacity(range_count as usize);
        for _ in 0..range_count {
            ranges.push(SubsegmentIndexBoxSubsegmentRange::deserialize(&mut reader)?);
        }

        Ok(SubsegmentIndexBoxSubsegment { range_count, ranges })
    }
}

impl Serialize for SubsegmentIndexBoxSubsegment {
    fn serialize<W>(&self, mut writer: W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        self.range_count.serialize(&mut writer)?;

        for range in &self.ranges {
            range.serialize(&mut writer)?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct SubsegmentIndexBoxSubsegmentRange {
    pub level: u8,
    pub range_size: U24Be,
}

impl<'a> Deserialize<'a> for SubsegmentIndexBoxSubsegmentRange {
    fn deserialize<R>(mut reader: R) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let level = u8::deserialize(&mut reader)?;
        let range_size = U24Be::deserialize(&mut reader)?;

        Ok(SubsegmentIndexBoxSubsegmentRange { level, range_size })
    }
}

impl Serialize for SubsegmentIndexBoxSubsegmentRange {
    fn serialize<W>(&self, mut writer: W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        self.level.serialize(&mut writer)?;
        self.range_size.serialize(&mut writer)?;
        Ok(())
    }
}

/// Producer reference time box
///
/// ISO/IEC 14496-12 - 8.16.5
#[derive(Debug, IsoBox)]
#[iso_box(box_type = b"prft", skip_impl(deserialize_seed), crate_path = crate)]
pub struct ProducerReferenceTimeBox {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    pub reference_track_id: u32,
    pub ntp_timestamp: u64,
    pub media_time: u64,
}

impl<'a> DeserializeSeed<'a, FullBoxHeader> for ProducerReferenceTimeBox {
    fn deserialize_seed<R>(mut reader: R, seed: FullBoxHeader) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let reference_track_id = u32::deserialize(&mut reader)?;
        let ntp_timestamp = u64::deserialize(&mut reader)?;
        let media_time = if seed.version == 0 {
            u32::deserialize(&mut reader)? as u64
        } else {
            u64::deserialize(&mut reader)?
        };

        Ok(ProducerReferenceTimeBox {
            header: seed,
            reference_track_id,
            ntp_timestamp,
            media_time,
        })
    }
}
