use scuffle_bytes_util::IoResultExt;
use scuffle_bytes_util::zero_copy::{Deserialize, DeserializeSeed};

use crate::{Base64String, BoxHeader, FullBoxHeader, IsoBox, Utf8String};

/// FD item information box
///
/// ISO/IEC 14996-12 - 8.13.2
#[derive(Debug)]
pub struct FDItemInformationBox {
    pub header: FullBoxHeader,
    pub entry_count: u16,
    pub partition_entries: Vec<PartitionEntry>,
    pub session_info: Option<FDSessionGroupBox>,
    pub group_id_to_name: Option<GroupIdToNameBox>,
}

impl IsoBox for FDItemInformationBox {
    type Header = FullBoxHeader;

    const TYPE: [u8; 4] = *b"fiin";
}

impl<'a> Deserialize<'a> for FDItemInformationBox {
    fn deserialize<R>(mut reader: R) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let header = BoxHeader::deserialize(&mut reader)?;
        let header = FullBoxHeader::deserialize_seed(&mut reader, header)?;
        Self::deserialize_seed(&mut reader, header)
    }
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

#[derive(Debug, IsoBox)]
#[iso_box(box_type = b"paen", crate_path = crate)]
pub struct PartitionEntry {
    #[iso_box(header)]
    pub header: BoxHeader,
    #[iso_box(nested_box)]
    pub blocks_and_symbols: FilePartitionBox,
    #[iso_box(nested_box(collect))]
    pub fec_symbol_locations: Option<FECReservoirBox>,
    #[iso_box(nested_box(collect))]
    pub file_symbol_locations: Option<FileReservoirBox>,
}

/// File partition box
///
/// ISO/IEC 14996-12 - 8.13.3
#[derive(Debug)]
pub struct FilePartitionBox {
    pub header: FullBoxHeader,
    pub item_id: u32,
    pub packet_payload_size: u16,
    pub fec_encoding_id: u8,
    pub fec_instance_id: u16,
    pub max_source_block_length: u16,
    pub encoding_symbol_length: u16,
    pub max_number_of_encoding_symbols: u16,
    pub scheme_specific_info: Base64String,
    pub entry_count: u32,
    pub entries: Vec<FilePartitionBoxEntry>,
}

impl IsoBox for FilePartitionBox {
    type Header = FullBoxHeader;

    const TYPE: [u8; 4] = *b"fpar";
}

impl<'a> Deserialize<'a> for FilePartitionBox {
    fn deserialize<R>(mut reader: R) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let header = BoxHeader::deserialize(&mut reader)?;
        let header = FullBoxHeader::deserialize_seed(&mut reader, header)?;
        Self::deserialize_seed(&mut reader, header)
    }
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
        u8::deserialize(&mut reader)?; // reserved
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

/// FEC reservoir box
///
/// ISO/IEC 14996-12 - 8.13.4
#[derive(Debug)]
pub struct FECReservoirBox {
    pub header: FullBoxHeader,
    pub entry_count: u32,
    pub entries: Vec<FECReservoirBoxEntry>,
}

impl IsoBox for FECReservoirBox {
    type Header = FullBoxHeader;

    const TYPE: [u8; 4] = *b"fecr";
}

impl<'a> Deserialize<'a> for FECReservoirBox {
    fn deserialize<R>(mut reader: R) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let header = BoxHeader::deserialize(&mut reader)?;
        let header = FullBoxHeader::deserialize_seed(&mut reader, header)?;
        Self::deserialize_seed(&mut reader, header)
    }
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

/// FD session group box
///
/// ISO/IEC 14996-12 - 8.13.5
#[derive(Debug)]
pub struct FDSessionGroupBox {
    pub header: BoxHeader,
    pub num_session_groups: u16,
    pub session_groups: Vec<FDSessionGroupBoxSessionGroup>,
}

impl IsoBox for FDSessionGroupBox {
    type Header = BoxHeader;

    const TYPE: [u8; 4] = *b"segr";
}

impl<'a> Deserialize<'a> for FDSessionGroupBox {
    fn deserialize<R>(mut reader: R) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let header = BoxHeader::deserialize(&mut reader)?;
        Self::deserialize_seed(&mut reader, header)
    }
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

/// Group ID to name box
///
/// ISO/IEC 14996-12 - 8.13.6
#[derive(Debug)]
pub struct GroupIdToNameBox {
    pub header: FullBoxHeader,
    pub entry_count: u16,
    pub entries: Vec<GroupIdToNameBoxEntry>,
}

impl IsoBox for GroupIdToNameBox {
    type Header = FullBoxHeader;

    const TYPE: [u8; 4] = *b"gitn";
}

impl<'a> Deserialize<'a> for GroupIdToNameBox {
    fn deserialize<R>(mut reader: R) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let header = BoxHeader::deserialize(&mut reader)?;
        let header = FullBoxHeader::deserialize_seed(&mut reader, header)?;
        Self::deserialize_seed(&mut reader, header)
    }
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

/// File reservoir box
///
/// ISO/IEC 14996-12 - 8.13.7
#[derive(Debug)]
pub struct FileReservoirBox {
    pub header: FullBoxHeader,
    pub entry_count: u32,
    pub entries: Vec<FECReservoirBoxEntry>,
}

impl IsoBox for FileReservoirBox {
    type Header = FullBoxHeader;

    const TYPE: [u8; 4] = *b"fire";
}

impl<'a> Deserialize<'a> for FileReservoirBox {
    fn deserialize<R>(mut reader: R) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let header = BoxHeader::deserialize(&mut reader)?;
        let header = FullBoxHeader::deserialize_seed(&mut reader, header)?;
        Self::deserialize_seed(&mut reader, header)
    }
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
            entries.push(FECReservoirBoxEntry::deserialize_seed(&mut reader, seed.version)?);
        }

        Ok(Self {
            header: seed,
            entry_count,
            entries,
        })
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
