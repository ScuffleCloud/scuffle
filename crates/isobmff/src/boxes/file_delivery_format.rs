use scuffle_bytes_util::IoResultExt;
use scuffle_bytes_util::zero_copy::{Deserialize, DeserializeSeed, Serialize};

use crate::{Base64String, BoxHeader, FullBoxHeader, IsoBox, IsoSized, Utf8String};

/// FD item information box
///
/// ISO/IEC 14996-12 - 8.13.2
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"fiin", skip_impl(deserialize_seed, serialize), crate_path = crate)]
pub struct FDItemInformationBox {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// Provides a count of the number of entries in the [`partition_entries`](Self::partition_entries) vec.
    pub entry_count: u16,
    /// The contained partition entries.
    pub partition_entries: Vec<PartitionEntry>,
    /// The contained [`FDSessionGroupBox`]. (optional)
    pub session_info: Option<FDSessionGroupBox>,
    /// The contained [`GroupIdToNameBox`]. (optional)
    pub group_id_to_name: Option<GroupIdToNameBox>,
}

impl<'a> DeserializeSeed<'a, BoxHeader> for FDItemInformationBox {
    fn deserialize_seed<R>(mut reader: R, _seed: BoxHeader) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let full_header = FullBoxHeader::deserialize(&mut reader)?;
        let entry_count = u16::deserialize(&mut reader)?;

        let mut partition_entries = Vec::with_capacity(entry_count as usize);
        for _ in 0..entry_count {
            partition_entries.push(PartitionEntry::deserialize(&mut reader)?);
        }

        let session_info = FDSessionGroupBox::deserialize(&mut reader).eof_to_none()?;
        let group_id_to_name = GroupIdToNameBox::deserialize(&mut reader).eof_to_none()?;

        Ok(Self {
            full_header,
            entry_count,
            partition_entries,
            session_info,
            group_id_to_name,
        })
    }
}

impl Serialize for FDItemInformationBox {
    fn serialize<W>(&self, mut writer: W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        self.serialize_box_header(&mut writer)?;
        self.full_header.serialize(&mut writer)?;
        self.entry_count.serialize(&mut writer)?;

        for entry in &self.partition_entries {
            entry.serialize(&mut writer)?;
        }

        if let Some(ref session_info) = self.session_info {
            session_info.serialize(&mut writer)?;
        }

        if let Some(ref group_id_to_name) = self.group_id_to_name {
            group_id_to_name.serialize(&mut writer)?;
        }

        Ok(())
    }
}

/// FD item information partition entry
///
/// ISO/IEC 14996-12 - 8.13.2
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"paen", crate_path = crate)]
pub struct PartitionEntry {
    /// The contained [`FileReservoirBox`]. (optional)
    #[iso_box(nested_box(collect))]
    pub file_symbol_locations: Option<FileReservoirBox>,
    /// The contained [`FilePartitionBox`]. (mandatory)
    #[iso_box(nested_box)]
    pub blocks_and_symbols: FilePartitionBox,
    /// The contained [`FECReservoirBox`]. (optional)
    #[iso_box(nested_box(collect))]
    pub fec_symbol_locations: Option<FECReservoirBox>,
}

/// File partition box
///
/// ISO/IEC 14996-12 - 8.13.3
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"fpar", skip_impl(deserialize_seed, serialize, sized), crate_path = crate)]
pub struct FilePartitionBox {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// References the item in the [`ItemLocationBox`](super::ItemLocationBox) that the file partitioning applies to.
    pub item_id: u32,
    /// Gives the target ALC/LCT or FLUTE packet payload size of the partitioning
    /// algorithm. Note that UDP packet payloads are larger, as they also contain ALC/LCT or FLUTE
    /// headers.
    pub packet_payload_size: u16,
    /// Reserved 8 bits, must be set to 0.
    pub reserved: u8,
    /// Identifies the FEC encoding scheme using a "Reliable Multicast Transport
    /// (RMT) FEC Encoding ID" declared at IANA, as defined in IETF RFC 5052. Note that i) value zero
    /// corresponds to the "Compact No-Code FEC scheme" also known as "Null-FEC" (IETF RFC 3695);
    /// ii) value one corresponds to the “MBMS FEC” (3GPP TS 26.346); iii) for values in the range of 0
    /// to 127, inclusive, the FEC scheme is Fully-Specified, whereas for values in the range of 128 to 255,
    /// inclusive, the FEC scheme is Under-Specified.
    pub fec_encoding_id: u8,
    /// Provides a more specific identification of the FEC encoder being used for an
    /// Under-Specified FEC scheme. This value should be set to zero for Fully-Specified FEC schemes and
    /// shall be ignored when parsing a file with `FEC_encoding_ID` in the range of 0 to 127, inclusive.
    /// `FEC_instance_ID` is scoped by the `FEC_encoding_ID`. See IETF RFC 5052 for further details.
    pub fec_instance_id: u16,
    /// Gives the maximum number of source symbols per source block.
    pub max_source_block_length: u16,
    /// Gives the size (in bytes) of one encoding symbol. All encoding symbols of one
    /// item have the same length, except the last symbol which may be shorter.
    pub encoding_symbol_length: u16,
    /// Gives the maximum number of encoding symbols that can be
    /// generated for a source block for those FEC schemes in which the maximum number of encoding
    /// symbols is relevant, such as FEC encoding ID 129 defined in IETF RFC 5052. For those FEC schemes
    /// in which the maximum number of encoding symbols is not relevant, the semantics of this field is
    /// unspecified.
    pub max_number_of_encoding_symbols: u16,
    /// The scheme-specific object transfer information (FEC-OTI-Scheme-Specific-Info).
    /// The definition of the information depends on the FEC encoding ID.
    pub scheme_specific_info: Base64String,
    /// Gives the number of entries in the list of (`block_count`, `block_size`) pairs that provides a
    /// partitioning of the source file. Starting from the beginning of the file, each entry indicates how the
    /// next segment of the file is divided into source blocks and source symbols.
    pub entry_count: u32,
    /// `block_count` and `block_size` pairs.
    pub entries: Vec<FilePartitionBoxEntry>,
}

impl<'a> DeserializeSeed<'a, BoxHeader> for FilePartitionBox {
    fn deserialize_seed<R>(mut reader: R, _seed: BoxHeader) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let full_header = FullBoxHeader::deserialize(&mut reader)?;
        let item_id = if full_header.version == 0 {
            u16::deserialize(&mut reader)? as u32
        } else {
            u32::deserialize(&mut reader)?
        };
        let packet_payload_size = u16::deserialize(&mut reader)?;
        let reserved = u8::deserialize(&mut reader)?;
        let fec_encoding_id = u8::deserialize(&mut reader)?;
        let fec_instance_id = u16::deserialize(&mut reader)?;
        let max_source_block_length = u16::deserialize(&mut reader)?;
        let encoding_symbol_length = u16::deserialize(&mut reader)?;
        let max_number_of_encoding_symbols = u16::deserialize(&mut reader)?;
        let scheme_specific_info = Base64String::deserialize(&mut reader)?;

        let entry_count = if full_header.version == 0 {
            u16::deserialize(&mut reader)? as u32
        } else {
            u32::deserialize(&mut reader)?
        };

        let mut entries = Vec::with_capacity(entry_count as usize);
        for _ in 0..entry_count {
            entries.push(FilePartitionBoxEntry::deserialize(&mut reader)?);
        }

        Ok(Self {
            full_header,
            item_id,
            packet_payload_size,
            reserved,
            fec_encoding_id,
            fec_instance_id,
            max_source_block_length,
            encoding_symbol_length,
            max_number_of_encoding_symbols,
            scheme_specific_info,
            entry_count,
            entries,
        })
    }
}

impl Serialize for FilePartitionBox {
    fn serialize<W>(&self, mut writer: W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        self.serialize_box_header(&mut writer)?;
        self.full_header.serialize(&mut writer)?;
        if self.full_header.version == 0 {
            (self.item_id as u16).serialize(&mut writer)?;
        } else {
            self.item_id.serialize(&mut writer)?;
        }
        self.packet_payload_size.serialize(&mut writer)?;
        self.reserved.serialize(&mut writer)?;
        self.fec_encoding_id.serialize(&mut writer)?;
        self.fec_instance_id.serialize(&mut writer)?;
        self.max_source_block_length.serialize(&mut writer)?;
        self.encoding_symbol_length.serialize(&mut writer)?;
        self.max_number_of_encoding_symbols.serialize(&mut writer)?;
        self.scheme_specific_info.serialize(&mut writer)?;

        if self.full_header.version == 0 {
            (self.entry_count as u16).serialize(&mut writer)?;
        } else {
            self.entry_count.serialize(&mut writer)?;
        }

        for entry in &self.entries {
            entry.serialize(&mut writer)?;
        }

        Ok(())
    }
}

impl IsoSized for FilePartitionBox {
    fn size(&self) -> usize {
        let mut size = self.full_header.size();
        if self.full_header.version == 0 {
            size += 2; // item_id
        } else {
            size += 4; // item_id
        }
        size += self.packet_payload_size.size()
            + self.reserved.size()
            + self.fec_encoding_id.size()
            + self.fec_instance_id.size()
            + self.max_source_block_length.size()
            + self.encoding_symbol_length.size()
            + self.max_number_of_encoding_symbols.size()
            + self.scheme_specific_info.size();
        if self.full_header.version == 0 {
            size += 2; // entry_count
        } else {
            size += 4; // entry_count
        }
        size += self.entries.size();

        Self::add_header_size(size)
    }
}

/// Entry in the [`FilePartitionBox`].
#[derive(Debug, PartialEq, Eq)]
pub struct FilePartitionBoxEntry {
    /// Indicates the number of consecutive source blocks of size `block_size`.
    pub block_count: u16,
    /// Indicates the size of a block (in bytes). A `block_size` that is not a multiple of the
    /// `encoding_symbol_length` symbol size indicates with Compact No-Code FEC that the last source symbols
    /// includes padding that is not stored in the item. With MBMS FEC (3GPP TS 26.346) the padding
    /// may extend across multiple symbols but the size of padding should never be more than
    /// `encoding_symbol_length`.
    pub block_size: u32,
}

impl<'a> Deserialize<'a> for FilePartitionBoxEntry {
    fn deserialize<R>(mut reader: R) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let block_count = u16::deserialize(&mut reader)?;
        let block_size = u32::deserialize(&mut reader)?;

        Ok(Self { block_count, block_size })
    }
}

impl Serialize for FilePartitionBoxEntry {
    fn serialize<W>(&self, mut writer: W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        self.block_count.serialize(&mut writer)?;
        self.block_size.serialize(&mut writer)?;
        Ok(())
    }
}

impl IsoSized for FilePartitionBoxEntry {
    fn size(&self) -> usize {
        self.block_count.size() + self.block_size.size()
    }
}

/// FEC reservoir box
///
/// ISO/IEC 14996-12 - 8.13.4
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"fecr", skip_impl(deserialize_seed, serialize, sized), crate_path = crate)]
pub struct FECReservoirBox {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// Gives the number of entries in the [`entries`](Self::entries) vec. An entry count here should match the
    /// total number of blocks in the corresponding [`FilePartitionBox`].
    pub entry_count: u32,
    /// The contained entries.
    pub entries: Vec<FECReservoirBoxEntry>,
}

impl<'a> DeserializeSeed<'a, BoxHeader> for FECReservoirBox {
    fn deserialize_seed<R>(mut reader: R, _seed: BoxHeader) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let full_header = FullBoxHeader::deserialize(&mut reader)?;
        let entry_count = if full_header.version == 0 {
            u16::deserialize(&mut reader)? as u32
        } else {
            u32::deserialize(&mut reader)?
        };

        let mut entries = Vec::with_capacity(entry_count as usize);
        for _ in 0..entry_count {
            entries.push(FECReservoirBoxEntry::deserialize_seed(&mut reader, full_header.version)?);
        }

        Ok(Self {
            full_header,
            entry_count,
            entries,
        })
    }
}

impl Serialize for FECReservoirBox {
    fn serialize<W>(&self, mut writer: W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        self.serialize_box_header(&mut writer)?;
        self.full_header.serialize(&mut writer)?;
        if self.full_header.version == 0 {
            (self.entry_count as u16).serialize(&mut writer)?;
        } else {
            self.entry_count.serialize(&mut writer)?;
        }

        for entry in &self.entries {
            entry.serialize(&mut writer, self.full_header.version)?;
        }

        Ok(())
    }
}

impl IsoSized for FECReservoirBox {
    fn size(&self) -> usize {
        let mut size = self.full_header.size();
        if self.full_header.version == 0 {
            size += (self.entry_count as u16).size();
        } else {
            size += self.entry_count.size();
        }
        size += self
            .entries
            .iter()
            .map(|entry| entry.size(self.full_header.version))
            .sum::<usize>();

        Self::add_header_size(size)
    }
}

/// Entry in the [`FECReservoirBox`].
#[derive(Debug, PartialEq, Eq)]
pub struct FECReservoirBoxEntry {
    /// Indicates the location of the FEC reservoir associated with a source block.
    pub item_id: u32,
    /// Indicates the number of repair symbols contained in the FEC reservoir.
    pub symbol_count: u32,
}

impl<'a> DeserializeSeed<'a, u8> for FECReservoirBoxEntry {
    fn deserialize_seed<R>(mut reader: R, seed: u8) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let item_id = if seed == 0 {
            u16::deserialize(&mut reader)? as u32
        } else {
            u32::deserialize(&mut reader)?
        };
        let symbol_count = u32::deserialize(&mut reader)?;

        Ok(Self { item_id, symbol_count })
    }
}

impl FECReservoirBoxEntry {
    fn serialize<W>(&self, mut writer: W, version: u8) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        if version == 0 {
            (self.item_id as u16).serialize(&mut writer)?;
        } else {
            self.item_id.serialize(&mut writer)?;
        }
        self.symbol_count.serialize(&mut writer)?;

        Ok(())
    }
}

impl FECReservoirBoxEntry {
    /// Returns the size of the entry, depending on the version.
    pub fn size(&self, version: u8) -> usize {
        if version == 0 {
            (self.item_id as u16).size() + self.symbol_count.size()
        } else {
            self.item_id.size() + self.symbol_count.size()
        }
    }
}

/// FD session group box
///
/// ISO/IEC 14996-12 - 8.13.5
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"segr", skip_impl(deserialize_seed, serialize), crate_path = crate)]
pub struct FDSessionGroupBox {
    /// Specifies the number of session groups.
    pub num_session_groups: u16,
    /// The contained session groups.
    pub session_groups: Vec<FDSessionGroupBoxSessionGroup>,
}

impl<'a> DeserializeSeed<'a, BoxHeader> for FDSessionGroupBox {
    fn deserialize_seed<R>(mut reader: R, _seed: BoxHeader) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let num_session_groups = u16::deserialize(&mut reader)?;

        let mut session_groups = Vec::with_capacity(num_session_groups as usize);
        for _ in 0..num_session_groups {
            session_groups.push(FDSessionGroupBoxSessionGroup::deserialize(&mut reader)?);
        }

        Ok(Self {
            num_session_groups,
            session_groups,
        })
    }
}

impl Serialize for FDSessionGroupBox {
    fn serialize<W>(&self, mut writer: W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        self.serialize_box_header(&mut writer)?;
        self.num_session_groups.serialize(&mut writer)?;

        for group in &self.session_groups {
            group.serialize(&mut writer)?;
        }

        Ok(())
    }
}

/// Session group in the [`FDSessionGroupBox`].
#[derive(Debug, PartialEq, Eq)]
pub struct FDSessionGroupBoxSessionGroup {
    /// Gives the number of entries in the following list comprising all file groups that the session
    /// group complies with. The session group contains all files included in the listed file groups as
    /// specified by the item information entry of each source file. The FDT for the session group should
    /// only contain those groups that are listed in this structure.
    pub entry_count: u8,
    /// Indicates a file group that the session group complies with.
    pub group_id: Vec<u32>,
    /// Specifies the number of channels in the session group.
    /// The value of `num_channels_in_session_groups` shall be a positive integer.
    pub num_channels_in_session_group: u16,
    /// Specifies the track identifier of the FD hint track belonging to a particular session group.
    /// Note that one FD hint track corresponds to one LCT channel.
    pub hint_track_id: Vec<u32>,
}

impl<'a> Deserialize<'a> for FDSessionGroupBoxSessionGroup {
    fn deserialize<R>(mut reader: R) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let entry_count = u8::deserialize(&mut reader)?;
        let mut group_id = Vec::with_capacity(entry_count as usize);
        for _ in 0..entry_count {
            group_id.push(u32::deserialize(&mut reader)?);
        }

        let num_channels_in_session_group = u16::deserialize(&mut reader)?;
        let mut hint_track_id = Vec::with_capacity(entry_count as usize);
        for _ in 0..entry_count {
            hint_track_id.push(u32::deserialize(&mut reader)?);
        }

        Ok(Self {
            entry_count,
            group_id,
            num_channels_in_session_group,
            hint_track_id,
        })
    }
}

impl Serialize for FDSessionGroupBoxSessionGroup {
    fn serialize<W>(&self, mut writer: W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        self.entry_count.serialize(&mut writer)?;
        for id in &self.group_id {
            id.serialize(&mut writer)?;
        }

        self.num_channels_in_session_group.serialize(&mut writer)?;
        for id in &self.hint_track_id {
            id.serialize(&mut writer)?;
        }

        Ok(())
    }
}

impl IsoSized for FDSessionGroupBoxSessionGroup {
    fn size(&self) -> usize {
        self.entry_count.size()
            + self.group_id.size()
            + self.num_channels_in_session_group.size()
            + self.hint_track_id.size()
    }
}

/// Group ID to name box
///
/// ISO/IEC 14996-12 - 8.13.6
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"gitn", skip_impl(deserialize_seed, serialize), crate_path = crate)]
pub struct GroupIdToNameBox {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// Gives the number of entries in the [`entries`](Self::entries) vec.
    pub entry_count: u16,
    /// The contained entries.
    pub entries: Vec<GroupIdToNameBoxEntry>,
}

impl<'a> DeserializeSeed<'a, BoxHeader> for GroupIdToNameBox {
    fn deserialize_seed<R>(mut reader: R, _seed: BoxHeader) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let full_header = FullBoxHeader::deserialize(&mut reader)?;
        let entry_count = u16::deserialize(&mut reader)?;

        let mut entries = Vec::with_capacity(entry_count as usize);
        for _ in 0..entry_count {
            entries.push(GroupIdToNameBoxEntry::deserialize(&mut reader)?);
        }

        Ok(Self {
            full_header,
            entry_count,
            entries,
        })
    }
}

impl Serialize for GroupIdToNameBox {
    fn serialize<W>(&self, mut writer: W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        self.serialize_box_header(&mut writer)?;
        self.full_header.serialize(&mut writer)?;
        self.entry_count.serialize(&mut writer)?;

        for entry in &self.entries {
            entry.serialize(&mut writer)?;
        }

        Ok(())
    }
}

/// Entry in the [`GroupIdToNameBox`].
#[derive(Debug, PartialEq, Eq)]
pub struct GroupIdToNameBoxEntry {
    /// Indicates a file group.
    pub group_id: u32,
    /// The file group name.
    pub group_name: Utf8String,
}

impl<'a> Deserialize<'a> for GroupIdToNameBoxEntry {
    fn deserialize<R>(mut reader: R) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        Ok(Self {
            group_id: u32::deserialize(&mut reader)?,
            group_name: Utf8String::deserialize(&mut reader)?,
        })
    }
}

impl Serialize for GroupIdToNameBoxEntry {
    fn serialize<W>(&self, mut writer: W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        self.group_id.serialize(&mut writer)?;
        self.group_name.serialize(&mut writer)?;
        Ok(())
    }
}

impl IsoSized for GroupIdToNameBoxEntry {
    fn size(&self) -> usize {
        self.group_id.size() + self.group_name.size()
    }
}

/// File reservoir box
///
/// ISO/IEC 14996-12 - 8.13.7
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"fire", skip_impl(deserialize_seed, serialize, sized), crate_path = crate)]
pub struct FileReservoirBox {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// Gives the number of entries in the [`entries`](Self::entries) vec. An entry count here should match the
    /// total number or blocks in the corresponding [`FilePartitionBox`].
    pub entry_count: u32,
    /// The contained entries.
    pub entries: Vec<FileReservoirBoxEntry>,
}

impl<'a> DeserializeSeed<'a, BoxHeader> for FileReservoirBox {
    fn deserialize_seed<R>(mut reader: R, _seed: BoxHeader) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let full_header = FullBoxHeader::deserialize(&mut reader)?;
        let entry_count = if full_header.version == 0 {
            u16::deserialize(&mut reader)? as u32
        } else {
            u32::deserialize(&mut reader)?
        };

        let mut entries = Vec::with_capacity(entry_count as usize);
        for _ in 0..entry_count {
            entries.push(FileReservoirBoxEntry::deserialize_seed(&mut reader, full_header.version)?);
        }

        Ok(Self {
            full_header,
            entry_count,
            entries,
        })
    }
}

impl Serialize for FileReservoirBox {
    fn serialize<W>(&self, mut writer: W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        self.serialize_box_header(&mut writer)?;
        self.full_header.serialize(&mut writer)?;
        if self.full_header.version == 0 {
            (self.entry_count as u16).serialize(&mut writer)?;
        } else {
            self.entry_count.serialize(&mut writer)?;
        }

        for entry in &self.entries {
            entry.serialize(&mut writer, self.full_header.version)?;
        }

        Ok(())
    }
}

impl IsoSized for FileReservoirBox {
    fn size(&self) -> usize {
        let mut size = self.full_header.size();
        if self.full_header.version == 0 {
            size += (self.entry_count as u16).size();
        } else {
            size += self.entry_count.size();
        }
        size += self
            .entries
            .iter()
            .map(|entry| entry.size(self.full_header.version))
            .sum::<usize>();

        Self::add_header_size(size)
    }
}

/// Entry in the [`FileReservoirBox`].
#[derive(Debug, PartialEq, Eq)]
pub struct FileReservoirBoxEntry {
    /// Indicates the location of the File reservoir associated with a source block.
    pub item_id: u32,
    /// Indicates the number of source symbols contained in the file reservoir.
    pub symbol_count: u32,
}

impl<'a> DeserializeSeed<'a, u8> for FileReservoirBoxEntry {
    fn deserialize_seed<R>(mut reader: R, seed: u8) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let item_id = if seed == 0 {
            u16::deserialize(&mut reader)? as u32
        } else {
            u32::deserialize(&mut reader)?
        };
        let symbol_count = u32::deserialize(&mut reader)?;

        Ok(Self { item_id, symbol_count })
    }
}

impl FileReservoirBoxEntry {
    fn serialize<W>(&self, mut writer: W, version: u8) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        if version == 0 {
            (self.item_id as u16).serialize(&mut writer)?;
        } else {
            self.item_id.serialize(&mut writer)?;
        }
        self.symbol_count.serialize(&mut writer)?;

        Ok(())
    }
}

impl FileReservoirBoxEntry {
    /// Returns the size of the entry, depending on the version.
    pub fn size(&self, version: u8) -> usize {
        if version == 0 {
            (self.item_id as u16).size() + self.symbol_count.size()
        } else {
            self.item_id.size() + self.symbol_count.size()
        }
    }
}
