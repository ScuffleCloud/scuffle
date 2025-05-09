use scuffle_bytes_util::zero_copy::{Deserialize, DeserializeSeed, U24Be};

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
#[derive(Debug)]
pub struct SegmentIndexBox {
    pub header: FullBoxHeader,
    pub reference_id: u32,
    pub timescale: u32,
    pub earliest_presentation_time: u64,
    pub first_offset: u64,
    pub reference_count: u16,
    pub references: Vec<SegmentIndexBoxReference>,
}

impl IsoBox for SegmentIndexBox {
    type Header = FullBoxHeader;

    const TYPE: [u8; 4] = *b"sidx";
}

impl<'a> Deserialize<'a> for SegmentIndexBox {
    fn deserialize<R>(mut reader: R) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let header = BoxHeader::deserialize(&mut reader)?;
        let header = FullBoxHeader::deserialize_seed(&mut reader, header)?;
        Self::deserialize_seed(&mut reader, header)
    }
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

        u16::deserialize(&mut reader)?; // reserved
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
            reference_count,
            references,
        })
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

#[derive(Debug)]
pub struct SubsegmentIndexBoxSubsegmentRange {
    pub level: u8,
    pub range_size: u32,
}

impl<'a> Deserialize<'a> for SubsegmentIndexBoxSubsegmentRange {
    fn deserialize<R>(mut reader: R) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let level = u8::deserialize(&mut reader)?;
        let range_size = U24Be::deserialize(&mut reader)?.into();

        Ok(SubsegmentIndexBoxSubsegmentRange { level, range_size })
    }
}

/// Producer reference time box
///
/// ISO/IEC 14496-12 - 8.16.5
#[derive(Debug)]
pub struct ProducerReferenceTimeBox {
    pub header: FullBoxHeader,
    pub reference_track_id: u32,
    pub ntp_timestamp: u64,
    pub media_time: u64,
}

impl IsoBox for ProducerReferenceTimeBox {
    type Header = FullBoxHeader;

    const TYPE: [u8; 4] = *b"prft";
}

impl<'a> Deserialize<'a> for ProducerReferenceTimeBox {
    fn deserialize<R>(mut reader: R) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let header = BoxHeader::deserialize(&mut reader)?;
        let header = FullBoxHeader::deserialize_seed(&mut reader, header)?;
        Self::deserialize_seed(&mut reader, header)
    }
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
