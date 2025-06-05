use scuffle_bytes_util::BitWriter;
use scuffle_bytes_util::zero_copy::{Deserialize, DeserializeSeed, Serialize, U24Be};

use super::Brand;
use crate::{BoxHeader, FullBoxHeader, IsoBox, IsoSized};

/// Segment type box
///
/// ISO/IEC 14496-12 - 8.16.2
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"styp", crate_path = crate)]
pub struct SegmentTypeBox {
    /// The "best use" brand of the file which will provide the greatest compatibility.
    #[iso_box(from = "[u8; 4]")]
    pub major_brand: Brand,
    /// Minor version of the major brand.
    pub minor_version: u32,
    /// A list of compatible brands.
    #[iso_box(repeated, from = "[u8; 4]")]
    pub compatible_brands: Vec<Brand>,
}

/// Segment index box
///
/// ISO/IEC 14496-12 - 8.16.3
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"sidx", skip_impl(deserialize_seed, serialize, sized), crate_path = crate)]
pub struct SegmentIndexBox {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// Provides the stream ID for the reference stream; if this [`SegmentIndexBox`] is referenced
    /// from a "parent" [`SegmentIndexBox`], the value of `reference_ID` shall be the same as the value of
    /// reference_ID of the "parent" [`SegmentIndexBox`].
    pub reference_id: u32,
    /// Provides the timescale, in ticks per second, for the time and duration fields within this box;
    /// it is recommended that this match the timescale of the reference stream or track; for files based on
    /// this document, that is the timescale field of the media header box of the track.
    pub timescale: u32,
    /// The earliest presentation time of any content in the reference stream
    /// in the first subsegment, in the timescale indicated in the timescale field; the earliest presentation
    /// time is derived from media in access units, or parts of access units, that are not omitted by an edit
    /// list (if any).
    pub earliest_presentation_time: u64,
    /// The distance in bytes, in the file containing media, from the anchor point, to the first
    /// byte of the indexed material.
    pub first_offset: u64,
    /// Reserved 16 bits, must be set to 0.
    pub reserved: u16,
    /// Provides the number of [referenced items](Self::references).
    pub reference_count: u16,
    /// The referenced items.
    pub references: Vec<SegmentIndexBoxReference>,
}

impl<'a> DeserializeSeed<'a, BoxHeader> for SegmentIndexBox {
    fn deserialize_seed<R>(mut reader: R, _seed: BoxHeader) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let full_header = FullBoxHeader::deserialize(&mut reader)?;

        let reference_id = u32::deserialize(&mut reader)?;
        let timescale = u32::deserialize(&mut reader)?;

        let earliest_presentation_time = if full_header.version == 0 {
            u32::deserialize(&mut reader)? as u64
        } else {
            u64::deserialize(&mut reader)?
        };
        let first_offset = if full_header.version == 0 {
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
            full_header,
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
        self.serialize_box_header(&mut writer)?;
        self.full_header.serialize(&mut writer)?;

        self.reference_id.serialize(&mut writer)?;
        self.timescale.serialize(&mut writer)?;

        if self.full_header.version == 0 {
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

impl IsoSized for SegmentIndexBox {
    fn size(&self) -> usize {
        let mut size = self.full_header.size();
        size += 4; // reference_id
        size += 4; // timescale
        if self.full_header.version == 0 {
            size += 4; // earliest_presentation_time
            size += 4; // first_offset
        } else {
            size += 8; // earliest_presentation_time
            size += 8; // first_offset
        }
        size += 2; // reserved
        size += 2; // reference_count

        size += self.references.size();

        Self::add_header_size(size)
    }
}

/// Reference in a [`SegmentIndexBox`].
#[derive(Debug, PartialEq, Eq)]
pub struct SegmentIndexBoxReference {
    /// When set to 1 indicates that the reference is to a [`SegmentIndexBox`]; otherwise the
    /// reference is to media content (e.g., in the case of files based on this document, to a
    /// [`MovieFragmentBox`](super::MovieFragmentBox)).
    /// If a separate index segment is used, then entries with reference type 1 are in the index segment,
    /// and entries with reference type 0 are in the media file.
    pub reference_type: bool,
    /// The distance in bytes from the first byte of the referenced item to the first byte of the
    /// next referenced item, or in the case of the last entry, the end of the referenced material.
    pub referenced_size: u32,
    /// When the reference is to [`SegmentIndexBox`], this field carries the sum of the
    /// `subsegment_duration` fields in that box; when the reference is to a subsegment, this field carries
    /// the difference between the earliest presentation time of any access unit of the reference stream
    /// in the next subsegment (or the first subsegment of the next segment, if this is the last subsegment
    /// of the segment, or the end presentation time of the reference stream if this is the last subsegment
    /// of the stream) and the earliest presentation time of any access unit of the reference stream in the
    /// referenced subsegment; the duration is in the same units as `earliest_presentation_time`.
    pub subsegment_duration: u32,
    /// Indicates whether the referenced subsegments start with a SAP. For the detailed
    /// semantics of this field in combination with other fields, see Table 6.
    pub starts_with_sap: bool,
    /// Indicates a SAP type as specified in Annex I, or the value 0. Other type values are reserved.
    /// For the detailed semantics of this field in combination with other fields, see the table below.
    pub sap_type: u8,
    /// Indicates T_SAP of the first SAP, in decoding order, in the referenced subsegment for the
    /// reference stream. If the referenced subsegments do not contain a SAP, `SAP_delta_time` is reserved
    /// with the value 0; otherwise `SAP_delta_time` is the difference between the earliest presentation
    /// time of the subsegment, and the TSAP (this difference may be zero, in the case that the subsegment
    /// starts with a SAP).
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

impl IsoSized for SegmentIndexBoxReference {
    fn size(&self) -> usize {
        4 + 4 + 4 // 3 u32s
    }
}

/// Subsegment index box
///
/// ISO/IEC 14496-12 - 8.16.4
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"ssix", crate_path = crate)]
pub struct SubsegmentIndexBox {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// A positive integer specifying the number of subsegments for which partial
    /// subsegment information is specified in this box. `subsegment_count` shall be equal `toreference_count`
    /// (i.e., the number of movie fragment references) in the immediately preceding [`SegmentIndexBox`].
    pub subsegment_count: u32,
    /// Subsegments in this box.
    #[iso_box(repeated)]
    pub subsegments: Vec<SubsegmentIndexBoxSubsegment>,
}

/// Subsegment in [`SubsegmentIndexBox`].
#[derive(Debug, PartialEq, Eq)]
pub struct SubsegmentIndexBoxSubsegment {
    /// Specifies the number of partial subsegment levels into which the media data is grouped.
    /// This value shall be greater than or equal to 2.
    pub range_count: u32,
    /// `range_size` and `level`.
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

impl IsoSized for SubsegmentIndexBoxSubsegment {
    fn size(&self) -> usize {
        self.range_count.size() + self.ranges.size()
    }
}

/// Subsegment range in [`SubsegmentIndexBoxSubsegment`].
#[derive(Debug, PartialEq, Eq)]
pub struct SubsegmentIndexBoxSubsegmentRange {
    /// Specifies the level to which this partial subsegment is assigned.
    pub level: u8,
    /// Indicates the size of the partial subsegment; the value 0 may be used in the last entry to
    /// indicate the remaining bytes of the segment, to the end of the segment.
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

impl IsoSized for SubsegmentIndexBoxSubsegmentRange {
    fn size(&self) -> usize {
        self.level.size() + self.range_size.size()
    }
}

/// Producer reference time box
///
/// ISO/IEC 14496-12 - 8.16.5
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"prft", skip_impl(deserialize_seed, serialize, sized), crate_path = crate)]
pub struct ProducerReferenceTimeBox {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// Provides the track_ID for the reference track.
    pub reference_track_id: u32,
    /// Indicates a UTC time in NTP format associated to `media_time` as follows:
    ///
    /// - if `flags` is set to 0, the UTC time is the time at which the frame belonging to the reference track in
    ///   the following movie fragment and whose presentation time is `media_time` was input to the encoder.
    /// - if `flags` is set to 1, the UTC time is the time at which the frame belonging to the reference track in the
    ///   following movie fragment and whose presentation time is `media_time` was output from the encoder.
    /// - if `flags` is set to 2, the UTC time is the time at which the following
    ///   [`MovieFragmentBox`](super::MovieFragmentBox) was finalized.
    ///   `media_time` is set to the presentation of the earliest frame of the reference track in presentation
    ///   order of the movie fragment.
    /// - if `flags` is set to 4, the UTC time is the time at which the following [`MovieFragmentBox`](super::MovieFragmentBox)
    ///   was written to file.
    ///   `media_time` is set to the presentation of the earliest frame of the reference track in presentation
    ///   order of the movie fragment.
    /// - if `flags` is set to 8, the association between the `media_time` and UTC time is arbitrary but consistent
    ///   between multiple occurrences of this box in the same track.
    /// - if `flags` is set to 24 (i.e. the two bits corresponding to value 8 and 16 are set), the UTC time has a
    ///   consistent, small (ideally zero), offset from the real-time of the experience depicted in the media at
    ///   `media_time`.
    pub ntp_timestamp: u64,
    /// Expressed in the time units used for the reference track.
    pub media_time: u64,
}

impl<'a> DeserializeSeed<'a, BoxHeader> for ProducerReferenceTimeBox {
    fn deserialize_seed<R>(mut reader: R, _seed: BoxHeader) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let full_header = FullBoxHeader::deserialize(&mut reader)?;

        let reference_track_id = u32::deserialize(&mut reader)?;
        let ntp_timestamp = u64::deserialize(&mut reader)?;
        let media_time = if full_header.version == 0 {
            u32::deserialize(&mut reader)? as u64
        } else {
            u64::deserialize(&mut reader)?
        };

        Ok(ProducerReferenceTimeBox {
            full_header,
            reference_track_id,
            ntp_timestamp,
            media_time,
        })
    }
}

impl Serialize for ProducerReferenceTimeBox {
    fn serialize<W>(&self, mut writer: W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        self.serialize_box_header(&mut writer)?;
        self.full_header.serialize(&mut writer)?;

        self.reference_track_id.serialize(&mut writer)?;
        self.ntp_timestamp.serialize(&mut writer)?;
        if self.full_header.version == 0 {
            (self.media_time as u32).serialize(&mut writer)?;
        } else {
            self.media_time.serialize(&mut writer)?;
        }
        Ok(())
    }
}

impl IsoSized for ProducerReferenceTimeBox {
    fn size(&self) -> usize {
        let mut size = self.full_header.size();
        size += 4; // reference_track_id
        size += 8; // ntp_timestamp
        if self.full_header.version == 0 {
            size += 4; // media_time
        } else {
            size += 8; // media_time
        }

        Self::add_header_size(size)
    }
}
