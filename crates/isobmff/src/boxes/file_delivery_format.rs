use scuffle_bytes_util::IoResultExt;
use scuffle_bytes_util::zero_copy::{Deserialize, DeserializeSeed, Serialize, SerializeSeed};

use crate::{Base64String, BoxHeader, FullBoxHeader, IsoBox, IsoSized, Utf8String};

/// FD item information box
///
/// ISO/IEC 14996-12 - 8.13.2
#[derive(Debug, IsoBox)]
#[iso_box(box_type = b"fiin", skip_impl(deserialize_seed, serialize), crate_path = crate)]
pub struct FDItemInformationBox {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    pub entry_count: u16,
    pub partition_entries: Vec<PartitionEntry>,
    pub session_info: Option<FDSessionGroupBox>,
    pub group_id_to_name: Option<GroupIdToNameBox>,
}

impl<'a> DeserializeSeed<'a, FullBoxHeader> for FDItemInformationBox {
    fn deserialize_seed<R>(mut reader: R, seed: FullBoxHeader) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let entry_count = u16::deserialize(&mut reader)?;

        let mut partition_entries = Vec::with_capacity(entry_count as usize);
        for _ in 0..entry_count {
            partition_entries.push(PartitionEntry::deserialize(&mut reader)?);
        }

        let session_info = FDSessionGroupBox::deserialize(&mut reader).eof_to_none()?;
        let group_id_to_name = GroupIdToNameBox::deserialize(&mut reader).eof_to_none()?;

        Ok(Self {
            header: seed,
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
        self.header.serialize(&mut writer)?;
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

#[derive(Debug, IsoBox)]
#[iso_box(box_type = b"paen", crate_path = crate)]
pub struct PartitionEntry {
    #[iso_box(header)]
    pub header: BoxHeader,
    #[iso_box(nested_box(collect))]
    pub file_symbol_locations: Option<FileReservoirBox>,
    #[iso_box(nested_box)]
    pub blocks_and_symbols: FilePartitionBox,
    #[iso_box(nested_box(collect))]
    pub fec_symbol_locations: Option<FECReservoirBox>,
}

/// File partition box
///
/// ISO/IEC 14996-12 - 8.13.3
#[derive(Debug, IsoBox)]
#[iso_box(box_type = b"fpar", skip_impl(deserialize_seed, serialize, sized), crate_path = crate)]
pub struct FilePartitionBox {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    pub item_id: u32,
    pub packet_payload_size: u16,
    pub reserved: u8,
    pub fec_encoding_id: u8,
    pub fec_instance_id: u16,
    pub max_source_block_length: u16,
    pub encoding_symbol_length: u16,
    pub max_number_of_encoding_symbols: u16,
    pub scheme_specific_info: Base64String,
    pub entry_count: u32,
    pub entries: Vec<FilePartitionBoxEntry>,
}

impl<'a> DeserializeSeed<'a, FullBoxHeader> for FilePartitionBox {
    fn deserialize_seed<R>(mut reader: R, seed: FullBoxHeader) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let item_id = if seed.version == 0 {
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

        let entry_count = if seed.version == 0 {
            u16::deserialize(&mut reader)? as u32
        } else {
            u32::deserialize(&mut reader)?
        };

        let mut entries = Vec::with_capacity(entry_count as usize);
        for _ in 0..entry_count {
            entries.push(FilePartitionBoxEntry::deserialize(&mut reader)?);
        }

        Ok(Self {
            header: seed,
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
        self.header.serialize(&mut writer)?;
        if self.header.version == 0 {
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

        if self.header.version == 0 {
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
        let mut size = self.header.size();
        if self.header.version == 0 {
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
        if self.header.version == 0 {
            size += 2; // entry_count
        } else {
            size += 4; // entry_count
        }
        size += self.entries.size();
        size
    }
}

#[derive(Debug)]
pub struct FilePartitionBoxEntry {
    pub block_count: u16,
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
#[derive(Debug, IsoBox)]
#[iso_box(box_type = b"fecr", skip_impl(deserialize_seed, serialize, sized), crate_path = crate)]
pub struct FECReservoirBox {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    pub entry_count: u32,
    pub entries: Vec<FECReservoirBoxEntry>,
}

impl<'a> DeserializeSeed<'a, FullBoxHeader> for FECReservoirBox {
    fn deserialize_seed<R>(mut reader: R, seed: FullBoxHeader) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let entry_count = if seed.version == 0 {
            u16::deserialize(&mut reader)? as u32
        } else {
            u32::deserialize(&mut reader)?
        };

        let mut entries = Vec::with_capacity(entry_count as usize);
        for _ in 0..entry_count {
            entries.push(FECReservoirBoxEntry::deserialize_seed(&mut reader, seed.version)?);
        }

        Ok(Self {
            header: seed,
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
        self.header.serialize(&mut writer)?;
        if self.header.version == 0 {
            (self.entry_count as u16).serialize(&mut writer)?;
        } else {
            self.entry_count.serialize(&mut writer)?;
        }

        for entry in &self.entries {
            entry.serialize_seed(&mut writer, self.header.version)?;
        }

        Ok(())
    }
}

impl IsoSized for FECReservoirBox {
    fn size(&self) -> usize {
        let mut size = self.header.size();
        if self.header.version == 0 {
            size += (self.entry_count as u16).size();
        } else {
            size += self.entry_count.size();
        }
        size += self
            .entries
            .iter()
            .map(|entry| entry.size(self.header.version))
            .sum::<usize>();
        size
    }
}

#[derive(Debug)]
pub struct FECReservoirBoxEntry {
    pub item_id: u32,
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

impl SerializeSeed<u8> for FECReservoirBoxEntry {
    fn serialize_seed<W>(&self, mut writer: W, seed: u8) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        if seed == 0 {
            (self.item_id as u16).serialize(&mut writer)?;
        } else {
            self.item_id.serialize(&mut writer)?;
        }
        self.symbol_count.serialize(&mut writer)?;

        Ok(())
    }
}

impl FECReservoirBoxEntry {
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
#[derive(Debug, IsoBox)]
#[iso_box(box_type = b"segr", skip_impl(deserialize_seed, serialize), crate_path = crate)]
pub struct FDSessionGroupBox {
    #[iso_box(header)]
    pub header: BoxHeader,
    pub num_session_groups: u16,
    pub session_groups: Vec<FDSessionGroupBoxSessionGroup>,
}

impl<'a> DeserializeSeed<'a, BoxHeader> for FDSessionGroupBox {
    fn deserialize_seed<R>(mut reader: R, seed: BoxHeader) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let num_session_groups = u16::deserialize(&mut reader)?;

        let mut session_groups = Vec::with_capacity(num_session_groups as usize);
        for _ in 0..num_session_groups {
            session_groups.push(FDSessionGroupBoxSessionGroup::deserialize(&mut reader)?);
        }

        Ok(Self {
            header: seed,
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
        self.header.serialize(&mut writer)?;
        self.num_session_groups.serialize(&mut writer)?;

        for group in &self.session_groups {
            group.serialize(&mut writer)?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct FDSessionGroupBoxSessionGroup {
    pub entry_count: u8,
    pub group_id: Vec<u32>,
    pub num_channels_in_session_group: u16,
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
#[derive(Debug, IsoBox)]
#[iso_box(box_type = b"gitn", skip_impl(deserialize_seed, serialize), crate_path = crate)]
pub struct GroupIdToNameBox {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    pub entry_count: u16,
    pub entries: Vec<GroupIdToNameBoxEntry>,
}

impl<'a> DeserializeSeed<'a, FullBoxHeader> for GroupIdToNameBox {
    fn deserialize_seed<R>(mut reader: R, seed: FullBoxHeader) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let entry_count = u16::deserialize(&mut reader)?;

        let mut entries = Vec::with_capacity(entry_count as usize);
        for _ in 0..entry_count {
            entries.push(GroupIdToNameBoxEntry::deserialize(&mut reader)?);
        }

        Ok(Self {
            header: seed,
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
        self.header.serialize(&mut writer)?;
        self.entry_count.serialize(&mut writer)?;

        for entry in &self.entries {
            entry.serialize(&mut writer)?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct GroupIdToNameBoxEntry {
    pub group_id: u32,
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
#[derive(Debug, IsoBox)]
#[iso_box(box_type = b"fire", skip_impl(deserialize_seed, serialize, sized), crate_path = crate)]
pub struct FileReservoirBox {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    pub entry_count: u32,
    pub entries: Vec<FileReservoirBoxEntry>,
}

impl<'a> DeserializeSeed<'a, FullBoxHeader> for FileReservoirBox {
    fn deserialize_seed<R>(mut reader: R, seed: FullBoxHeader) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let entry_count = if seed.version == 0 {
            u16::deserialize(&mut reader)? as u32
        } else {
            u32::deserialize(&mut reader)?
        };

        let mut entries = Vec::with_capacity(entry_count as usize);
        for _ in 0..entry_count {
            entries.push(FileReservoirBoxEntry::deserialize_seed(&mut reader, seed.version)?);
        }

        Ok(Self {
            header: seed,
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
        self.header.serialize(&mut writer)?;
        if self.header.version == 0 {
            (self.entry_count as u16).serialize(&mut writer)?;
        } else {
            self.entry_count.serialize(&mut writer)?;
        }

        for entry in &self.entries {
            entry.serialize_seed(&mut writer, self.header.version)?;
        }

        Ok(())
    }
}

impl IsoSized for FileReservoirBox {
    fn size(&self) -> usize {
        let mut size = self.header.size();
        if self.header.version == 0 {
            size += (self.entry_count as u16).size();
        } else {
            size += self.entry_count.size();
        }
        size += self
            .entries
            .iter()
            .map(|entry| entry.size(self.header.version))
            .sum::<usize>();
        size
    }
}

#[derive(Debug)]
pub struct FileReservoirBoxEntry {
    pub item_id: u32,
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

impl SerializeSeed<u8> for FileReservoirBoxEntry {
    fn serialize_seed<W>(&self, mut writer: W, seed: u8) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        if seed == 0 {
            (self.item_id as u16).serialize(&mut writer)?;
        } else {
            self.item_id.serialize(&mut writer)?;
        }
        self.symbol_count.serialize(&mut writer)?;

        Ok(())
    }
}

impl FileReservoirBoxEntry {
    pub fn size(&self, version: u8) -> usize {
        if version == 0 {
            (self.item_id as u16).size() + self.symbol_count.size()
        } else {
            self.item_id.size() + self.symbol_count.size()
        }
    }
}
