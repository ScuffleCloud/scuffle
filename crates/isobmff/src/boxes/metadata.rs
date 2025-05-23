use std::io;

use scuffle_bytes_util::zero_copy::{Deserialize, DeserializeSeed, Serialize, SerializeSeed, U24Be};
use scuffle_bytes_util::{BitWriter, BytesCow, IoResultExt};

use super::{
    Brand, DataInformationBox, ExtendedTypeBox, FDItemInformationBox, HandlerBox, ProtectionSchemeInfoBox,
    ScrambleSchemeInfoBox,
};
use crate::{BoxHeader, FullBoxHeader, IsoBox, IsoSized, UnknownBox, Utf8String};

/// Meta box
///
/// ISO/IEC 14496-12 - 8.11.1
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"meta", crate_path = crate)]
pub struct MetaBox<'a> {
    pub full_header: FullBoxHeader,
    #[iso_box(nested_box)]
    pub hdlr: HandlerBox,
    #[iso_box(nested_box(collect))]
    pub dinf: Option<DataInformationBox<'a>>,
    #[iso_box(nested_box(collect))]
    pub iloc: Option<ItemLocationBox>,
    #[iso_box(nested_box(collect))]
    pub ipro: Option<ItemProtectionBox<'a>>,
    #[iso_box(nested_box(collect))]
    pub iinf: Option<ItemInfoBox<'a>>,
    #[iso_box(nested_box(collect))]
    pub xml: Option<XmlBox>,
    #[iso_box(nested_box(collect))]
    pub bxml: Option<BinaryXmlBox<'a>>,
    #[iso_box(nested_box(collect))]
    pub pitm: Option<PrimaryItemBox>,
    #[iso_box(nested_box(collect))]
    pub fiin: Option<FDItemInformationBox>,
    #[iso_box(nested_box(collect))]
    pub idat: Option<ItemDataBox<'a>>,
    #[iso_box(nested_box(collect))]
    pub iref: Option<ItemReferenceBox>,
    #[iso_box(nested_box(collect))]
    pub iprp: Option<ItemPropertiesBox<'a>>,
    #[iso_box(nested_box(collect_unknown))]
    pub unknown_boxes: Vec<UnknownBox<'a>>,
}

/// XML box
///
/// ISO/IEC 14496-12 - 8.11.2
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"xml ", crate_path = crate)]
pub struct XmlBox {
    pub full_header: FullBoxHeader,
    pub xml: Utf8String,
}

/// Binary XML box
///
/// ISO/IEC 14496-12 - 8.11.2
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"bxml", crate_path = crate)]
pub struct BinaryXmlBox<'a> {
    pub full_header: FullBoxHeader,
    pub data: BytesCow<'a>,
}

/// Item location box
///
/// ISO/IEC 14496-12 - 8.11.3
#[derive(Debug, IsoBox)]
#[iso_box(box_type = b"iloc", skip_impl(deserialize_seed, serialize, sized), crate_path = crate)]
pub struct ItemLocationBox {
    pub full_header: FullBoxHeader,
    pub offset_size: u8,
    pub length_size: u8,
    pub base_offset_size: u8,
    /// `index_size` or `reserved`
    pub index_size: u8,
    pub item_count: Option<u32>,
    pub items: Vec<ItemLocationBoxItem>,
}

impl<'a> DeserializeSeed<'a, BoxHeader> for ItemLocationBox {
    fn deserialize_seed<R>(mut reader: R, _seed: BoxHeader) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let full_header = FullBoxHeader::deserialize(&mut reader)?;

        let byte = u8::deserialize(&mut reader)?;
        let offset_size = byte >> 4;

        if ![0, 4, 8].contains(&offset_size) {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid offset_size"));
        }

        let length_size = byte & 0x0F;

        if ![0, 4, 8].contains(&length_size) {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid length_size"));
        }

        let byte = u8::deserialize(&mut reader)?;
        let base_offset_size = byte >> 4;

        if ![0, 4, 8].contains(&base_offset_size) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid base_offset_size",
            ));
        }

        let index_size = byte & 0x0F;
        if (full_header.version == 1 || full_header.version == 2) && ![0, 4, 8].contains(&index_size) {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid index_size"));
        }

        let item_count = if full_header.version < 2 {
            Some(u16::deserialize(&mut reader)? as u32)
        } else if full_header.version == 2 {
            Some(u32::deserialize(&mut reader)?)
        } else {
            None
        };

        let mut items = Vec::with_capacity(item_count.unwrap_or(0) as usize);
        for _ in 0..item_count.unwrap_or(0) {
            let item_id = if full_header.version < 2 {
                Some(u16::deserialize(&mut reader)? as u32)
            } else if full_header.version == 2 {
                Some(u32::deserialize(&mut reader)?)
            } else {
                None
            };

            let construction_method = if full_header.version == 1 || full_header.version == 2 {
                let value = u16::deserialize(&mut reader)?;
                Some((value & 0b1111) as u8)
            } else {
                None
            };

            let data_reference_index = u16::deserialize(&mut reader)?;
            let base_offset = reader.try_read(base_offset_size as usize)?.pad_to_u64_be();
            let extent_count = u16::deserialize(&mut reader)?;
            let mut extents = Vec::with_capacity(extent_count as usize);
            for _ in 0..extent_count {
                let item_reference_index = if (full_header.version == 1 || full_header.version == 2) && index_size > 0 {
                    Some(reader.try_read(index_size as usize)?.pad_to_u64_be())
                } else {
                    None
                };
                let extent_offset = reader.try_read(offset_size as usize)?.pad_to_u64_be();
                let extent_length = reader.try_read(length_size as usize)?.pad_to_u64_be();

                extents.push(ItemLocationBoxExtent {
                    item_reference_index,
                    extent_offset,
                    extent_length,
                });
            }

            items.push(ItemLocationBoxItem {
                item_id,
                construction_method,
                data_reference_index,
                base_offset,
                extent_count,
                extents,
            });
        }

        Ok(ItemLocationBox {
            full_header,
            offset_size,
            length_size,
            base_offset_size,
            index_size,
            item_count,
            items,
        })
    }
}

impl Serialize for ItemLocationBox {
    fn serialize<W>(&self, writer: W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        let mut bit_writer = BitWriter::new(writer);

        self.serialize_box_header(&mut bit_writer)?;
        self.full_header.serialize(&mut bit_writer)?;
        bit_writer.write_bits(self.offset_size as u64, 4)?;
        bit_writer.write_bits(self.length_size as u64, 4)?;
        bit_writer.write_bits(self.base_offset_size as u64, 4)?;
        bit_writer.write_bits(self.index_size as u64, 4)?;

        if self.full_header.version < 2 {
            (self
                .item_count
                .ok_or(io::Error::new(io::ErrorKind::InvalidData, "item_count is required"))? as u16)
                .serialize(&mut bit_writer)?;
        } else if self.full_header.version == 2 {
            self.item_count
                .ok_or(io::Error::new(io::ErrorKind::InvalidData, "item_count is required"))?
                .serialize(&mut bit_writer)?;
        }

        for item in &self.items {
            item.serialize_seed(&mut bit_writer, self)?;
        }

        Ok(())
    }
}

impl IsoSized for ItemLocationBox {
    fn size(&self) -> usize {
        let mut size = 0;

        size += self.full_header.size();
        size += 1; // offset_size + length_size
        size += 1; // base_offset_size + index_size/reserved

        if self.full_header.version < 2 {
            size += 2; // item_count
        } else if self.full_header.version == 2 {
            size += 4; // item_count
        }

        size += self.items.iter().map(|item| item.size(self)).sum::<usize>();

        Self::add_header_size(size)
    }
}

#[derive(Debug)]
pub struct ItemLocationBoxItem {
    pub item_id: Option<u32>,
    pub construction_method: Option<u8>,
    pub data_reference_index: u16,
    pub base_offset: u64,
    pub extent_count: u16,
    pub extents: Vec<ItemLocationBoxExtent>,
}

impl SerializeSeed<&ItemLocationBox> for ItemLocationBoxItem {
    fn serialize_seed<W>(&self, writer: W, seed: &ItemLocationBox) -> io::Result<()>
    where
        W: std::io::Write,
    {
        let mut bit_writer = BitWriter::new(writer);

        if seed.full_header.version < 2 {
            (self
                .item_id
                .ok_or(io::Error::new(io::ErrorKind::InvalidData, "item_id is required"))? as u16)
                .serialize(&mut bit_writer)?;
        } else if seed.full_header.version == 2 {
            self.item_id
                .ok_or(io::Error::new(io::ErrorKind::InvalidData, "item_id is required"))?
                .serialize(&mut bit_writer)?;
        }

        if seed.full_header.version == 1 || seed.full_header.version == 2 {
            bit_writer.write_bits(0, 12)?;
            bit_writer.write_bits(
                self.construction_method
                    .ok_or(io::Error::new(io::ErrorKind::InvalidData, "construction_method is required"))?
                    as u64,
                4,
            )?;
        }

        self.data_reference_index.serialize(&mut bit_writer)?;
        bit_writer.write_bits(self.base_offset, seed.base_offset_size * 8)?;
        self.extent_count.serialize(&mut bit_writer)?;

        for extent in &self.extents {
            extent.serialize_seed(&mut bit_writer, seed)?;
        }

        Ok(())
    }
}

impl ItemLocationBoxItem {
    pub fn size(&self, parent: &ItemLocationBox) -> usize {
        let mut size = 0;

        if parent.full_header.version < 2 {
            size += 2; // item_id
        } else if parent.full_header.version == 2 {
            size += 4; // item_id
        }
        if parent.full_header.version == 1 || parent.full_header.version == 2 {
            size += 2; // reserved + construction_method
        }
        size += 2; // data_reference_index
        size += parent.base_offset_size as usize; // base_offset
        size += 2; // extent_count
        size += self.extents.iter().map(|e| e.size(parent)).sum::<usize>();

        size
    }
}

#[derive(Debug)]
pub struct ItemLocationBoxExtent {
    pub item_reference_index: Option<u64>,
    pub extent_offset: u64,
    pub extent_length: u64,
}

impl SerializeSeed<&ItemLocationBox> for ItemLocationBoxExtent {
    fn serialize_seed<W>(&self, writer: W, seed: &ItemLocationBox) -> io::Result<()>
    where
        W: std::io::Write,
    {
        let mut bit_writer = BitWriter::new(writer);

        if (seed.full_header.version == 1 || seed.full_header.version == 2) && seed.index_size > 0 {
            bit_writer.write_bits(
                self.item_reference_index
                    .ok_or(io::Error::new(io::ErrorKind::InvalidData, "item_reference_index is required"))?,
                seed.index_size * 8,
            )?;
        }
        bit_writer.write_bits(self.extent_offset, seed.offset_size * 8)?;
        bit_writer.write_bits(self.extent_length, seed.length_size * 8)?;

        Ok(())
    }
}

impl ItemLocationBoxExtent {
    pub fn size(&self, parent: &ItemLocationBox) -> usize {
        let mut size = 0;

        if (parent.full_header.version == 1 || parent.full_header.version == 2) && parent.index_size > 0 {
            size += parent.index_size as usize; // item_reference_index
        }
        size += parent.offset_size as usize; // extent_offset
        size += parent.length_size as usize; // extent_length

        size
    }
}

/// Primary item box
///
/// ISO/IEC 14496-12 - 8.11.4
#[derive(Debug, IsoBox)]
#[iso_box(box_type = b"pitm", skip_impl(deserialize_seed, serialize, sized), crate_path = crate)]
pub struct PrimaryItemBox {
    pub full_header: FullBoxHeader,
    pub item_id: u32,
}

impl<'a> DeserializeSeed<'a, BoxHeader> for PrimaryItemBox {
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

        Ok(PrimaryItemBox { full_header, item_id })
    }
}

impl Serialize for PrimaryItemBox {
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

        Ok(())
    }
}

impl IsoSized for PrimaryItemBox {
    fn size(&self) -> usize {
        let mut size = self.full_header.size();

        if self.full_header.version == 0 {
            size += 2; // item_id
        } else {
            size += 4; // item_id
        }

        Self::add_header_size(size)
    }
}

/// Item protection box
///
/// ISO/IEC 14496-12 - 8.11.5
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"ipro", crate_path = crate)]
pub struct ItemProtectionBox<'a> {
    pub full_header: FullBoxHeader,
    pub protection_count: u16,
    #[iso_box(nested_box(collect))]
    pub protection_information: Vec<ProtectionSchemeInfoBox<'a>>,
}

/// Item information box
///
/// ISO/IEC 14496-12 - 8.11.6
#[derive(Debug, IsoBox)]
#[iso_box(box_type = b"iinf", skip_impl(deserialize_seed, serialize, sized), crate_path = crate)]
pub struct ItemInfoBox<'a> {
    pub full_header: FullBoxHeader,
    pub entry_count: u32,
    #[iso_box(nested_box(collect))]
    pub item_infos: Vec<ItemInfoEntry<'a>>,
}

impl<'a> DeserializeSeed<'a, BoxHeader> for ItemInfoBox<'a> {
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

        let mut item_infos = Vec::with_capacity(entry_count as usize);
        for _ in 0..entry_count {
            item_infos.push(ItemInfoEntry::deserialize(&mut reader)?);
        }

        Ok(ItemInfoBox {
            full_header,
            entry_count,
            item_infos,
        })
    }
}

impl Serialize for ItemInfoBox<'_> {
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

        for item_info in &self.item_infos {
            item_info.serialize(&mut writer)?;
        }

        Ok(())
    }
}

impl IsoSized for ItemInfoBox<'_> {
    fn size(&self) -> usize {
        let mut size = self.full_header.size();

        if self.full_header.version == 0 {
            size += 2; // entry_count
        } else {
            size += 4; // entry_count
        }

        size += self.item_infos.size();

        Self::add_header_size(size)
    }
}

/// Item information entry
///
/// ISO/IEC 14496-12 - 8.11.6
#[derive(Debug, IsoBox)]
#[iso_box(box_type = b"infe", skip_impl(deserialize_seed, serialize, sized), crate_path = crate)]
pub struct ItemInfoEntry<'a> {
    pub full_header: FullBoxHeader,
    pub item_id: Option<u32>,
    pub item_protection_index: u16,
    pub item_type: [u8; 4],
    pub item_name: Utf8String,
    pub item: Option<ItemInfoEntryItem>,
    pub extension_type: Option<[u8; 4]>,
    pub extension: Option<ItemInfoExtension<'a>>,
}

#[derive(Debug)]
pub enum ItemInfoEntryItem {
    Mime {
        content_type: Utf8String,
        content_encoding: Utf8String,
    },
    Uri {
        item_uri_type: Utf8String,
    },
}

impl<'a> DeserializeSeed<'a, BoxHeader> for ItemInfoEntry<'a> {
    fn deserialize_seed<R>(mut reader: R, _seed: BoxHeader) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let full_header = FullBoxHeader::deserialize(&mut reader)?;

        let item_id = if full_header.version == 0 || full_header.version == 1 || full_header.version == 2 {
            Some(u16::deserialize(&mut reader)? as u32)
        } else if full_header.version == 3 {
            Some(u32::deserialize(&mut reader)?)
        } else {
            None
        };
        let item_protection_index = u16::deserialize(&mut reader)?;
        let item_type = if full_header.version == 0 || full_header.version == 1 {
            *b"mime"
        } else {
            <[u8; 4]>::deserialize(&mut reader)?
        };
        let item_name = Utf8String::deserialize(&mut reader)?;

        let item = match &item_type {
            b"mime" => {
                let content_type = Utf8String::deserialize(&mut reader)?;
                let content_encoding = Utf8String::deserialize(&mut reader)?;

                Some(ItemInfoEntryItem::Mime {
                    content_type,
                    content_encoding,
                })
            }
            b"uri " => {
                let item_uri_type = Utf8String::deserialize(&mut reader)?;
                Some(ItemInfoEntryItem::Uri { item_uri_type })
            }
            _ => None,
        };

        let extension_type = if full_header.version == 1 {
            <[u8; 4]>::deserialize(&mut reader).eof_to_none()?
        } else {
            None
        };
        let extension = if let Some(extension_type) = extension_type {
            let extension = ItemInfoExtension::deserialize_seed(&mut reader, extension_type)?;
            Some(extension)
        } else {
            None
        };

        Ok(Self {
            full_header,
            item_id,
            item_protection_index,
            item_type,
            item_name,
            item,
            extension_type,
            extension,
        })
    }
}

impl Serialize for ItemInfoEntry<'_> {
    fn serialize<W>(&self, mut writer: W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        self.serialize_box_header(&mut writer)?;
        self.full_header.serialize(&mut writer)?;

        if self.full_header.version == 0 || self.full_header.version == 1 {
            (self
                .item_id
                .ok_or(io::Error::new(io::ErrorKind::InvalidData, "item_ID is required"))? as u16)
                .serialize(&mut writer)?;
            self.item_protection_index.serialize(&mut writer)?;
            self.item_name.serialize(&mut writer)?;
            if let Some(ItemInfoEntryItem::Mime {
                content_type,
                content_encoding,
            }) = self.item.as_ref()
            {
                content_type.serialize(&mut writer)?;
                content_encoding.serialize(&mut writer)?;
            } else {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "content_type is required"));
            }
        }

        if self.full_header.version == 1 {
            if let Some(extension_type) = self.extension_type {
                extension_type.serialize(&mut writer)?;
            }
            if let Some(extension) = self.extension.as_ref() {
                extension.serialize(&mut writer)?;
            }
        }

        if self.full_header.version >= 2 {
            if self.full_header.version == 2 {
                (self
                    .item_id
                    .ok_or(io::Error::new(io::ErrorKind::InvalidData, "item_ID is required"))? as u16)
                    .serialize(&mut writer)?;
            } else if self.full_header.version == 3 {
                self.item_id
                    .ok_or(io::Error::new(io::ErrorKind::InvalidData, "item_ID is required"))?
                    .serialize(&mut writer)?;
            }

            self.item_protection_index.serialize(&mut writer)?;
            self.item_type.serialize(&mut writer)?;
            self.item_name.serialize(&mut writer)?;
            match self
                .item
                .as_ref()
                .ok_or(io::Error::new(io::ErrorKind::InvalidData, "item is required"))?
            {
                ItemInfoEntryItem::Mime {
                    content_type,
                    content_encoding,
                } => {
                    content_type.serialize(&mut writer)?;
                    content_encoding.serialize(&mut writer)?;
                }
                ItemInfoEntryItem::Uri { item_uri_type } => {
                    item_uri_type.serialize(&mut writer)?;
                }
            }
        }

        Ok(())
    }
}

impl IsoSized for ItemInfoEntry<'_> {
    fn size(&self) -> usize {
        let mut size = self.full_header.size();

        if self.full_header.version == 0 || self.full_header.version == 1 {
            size += 2; // item_id
            size += 2; // item_protection_index
            size += self.item_name.size();
            if let Some(ItemInfoEntryItem::Mime {
                content_type,
                content_encoding,
            }) = &self.item
            {
                size += content_type.size();
                size += content_encoding.size();
            }
        }
        if self.full_header.version == 1 {
            size += self.extension_type.size();
            size += self.extension.size();
        }
        if self.full_header.version >= 2 {
            if self.full_header.version == 2 {
                size += 2; // item_id
            } else if self.full_header.version == 3 {
                size += 4; // item_id
            }
            size += 2; // item_protection_index
            size += self.item_type.size();
            size += self.item_name.size();
            match &self.item {
                Some(ItemInfoEntryItem::Mime {
                    content_type,
                    content_encoding,
                }) => {
                    size += content_type.size();
                    size += content_encoding.size();
                }
                Some(ItemInfoEntryItem::Uri { item_uri_type }) => {
                    size += item_uri_type.size();
                }
                None => {}
            }
        }

        Self::add_header_size(size)
    }
}

#[derive(Debug)]
pub enum ItemInfoExtension<'a> {
    // "fdel"
    FDItemInfoExtension {
        current_location: Utf8String,
        current_md5: Utf8String,
        content_length: u64,
        transfer_length: u64,
        entry_count: u8,
        group_id: Vec<u32>,
    },
    Other {
        extension_type: [u8; 4],
        data: BytesCow<'a>,
    },
}

impl<'a> DeserializeSeed<'a, [u8; 4]> for ItemInfoExtension<'a> {
    fn deserialize_seed<R>(mut reader: R, seed: [u8; 4]) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        match &seed {
            b"fdel" => {
                let current_location = Utf8String::deserialize(&mut reader)?;
                let current_md5 = Utf8String::deserialize(&mut reader)?;
                let content_length = u64::deserialize(&mut reader)?;
                let transfer_length = u64::deserialize(&mut reader)?;
                let entry_count = u8::deserialize(&mut reader)?;

                let mut group_id = Vec::with_capacity(entry_count as usize);
                for _ in 0..entry_count {
                    group_id.push(u32::deserialize(&mut reader)?);
                }

                Ok(ItemInfoExtension::FDItemInfoExtension {
                    current_location,
                    current_md5,
                    content_length,
                    transfer_length,
                    entry_count,
                    group_id,
                })
            }
            _ => Ok(ItemInfoExtension::Other {
                extension_type: seed,
                data: reader.try_read_to_end()?,
            }),
        }
    }
}

impl Serialize for ItemInfoExtension<'_> {
    fn serialize<W>(&self, mut writer: W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        match self {
            ItemInfoExtension::FDItemInfoExtension {
                current_location,
                current_md5,
                content_length,
                transfer_length,
                entry_count,
                group_id,
            } => {
                current_location.serialize(&mut writer)?;
                current_md5.serialize(&mut writer)?;
                content_length.serialize(&mut writer)?;
                transfer_length.serialize(&mut writer)?;
                entry_count.serialize(&mut writer)?;

                for id in group_id {
                    id.serialize(&mut writer)?;
                }

                Ok(())
            }
            ItemInfoExtension::Other { extension_type, data } => {
                extension_type.serialize(&mut writer)?;
                data.serialize(&mut writer)?;
                Ok(())
            }
        }
    }
}

impl IsoSized for ItemInfoExtension<'_> {
    fn size(&self) -> usize {
        match self {
            ItemInfoExtension::FDItemInfoExtension {
                current_location,
                current_md5,
                content_length,
                transfer_length,
                entry_count,
                group_id,
            } => {
                current_location.size()
                    + current_md5.size()
                    + content_length.size()
                    + transfer_length.size()
                    + entry_count.size()
                    + group_id.size()
            }
            ItemInfoExtension::Other { data, .. } => data.size(),
        }
    }
}

// 8.11.7 is deprecated

// 8.11.8 is deprecated

/// Item data box
///
/// ISO/IEC 14496-12 - 8.11.11
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"idat", crate_path = crate)]
pub struct ItemDataBox<'a> {
    pub data: BytesCow<'a>,
}

/// Item reference box
///
/// ISO/IEC 14496-12 - 8.11.12
#[derive(Debug, IsoBox)]
#[iso_box(box_type = b"iref", skip_impl(deserialize_seed), crate_path = crate)]
pub struct ItemReferenceBox {
    pub full_header: FullBoxHeader,
    #[iso_box(repeated)]
    pub references: Vec<SingleItemTypeReferenceBox>,
}

impl<'a> DeserializeSeed<'a, BoxHeader> for ItemReferenceBox {
    fn deserialize_seed<R>(mut reader: R, _seed: BoxHeader) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let full_header = FullBoxHeader::deserialize(&mut reader)?;

        let mut references = Vec::new();

        loop {
            let Some(header) = BoxHeader::deserialize(&mut reader).eof_to_none()? else {
                break;
            };

            if full_header.version == 0 {
                let Some(iso_box) =
                    SingleItemTypeReferenceBox::deserialize_seed(&mut reader, (header, None)).eof_to_none()?
                else {
                    break;
                };
                references.push(iso_box);
            } else if full_header.version == 1 {
                let Some(full_header) = FullBoxHeader::deserialize(&mut reader).eof_to_none()? else {
                    break;
                };
                let Some(iso_box) =
                    SingleItemTypeReferenceBox::deserialize_seed(&mut reader, (header, Some(full_header))).eof_to_none()?
                else {
                    break;
                };
                references.push(iso_box);
            }
        }

        Ok(ItemReferenceBox { full_header, references })
    }
}

#[derive(Debug)]
pub struct SingleItemTypeReferenceBox {
    pub header: BoxHeader,
    pub full_header: Option<FullBoxHeader>,
    pub from_item_id: u32,
    pub reference_count: u16,
    pub to_item_id: Vec<u32>,
}

impl<'a> DeserializeSeed<'a, (BoxHeader, Option<FullBoxHeader>)> for SingleItemTypeReferenceBox {
    fn deserialize_seed<R>(mut reader: R, seed: (BoxHeader, Option<FullBoxHeader>)) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        if seed.1.is_none() {
            let from_item_id = u16::deserialize(&mut reader)? as u32;
            let reference_count = u16::deserialize(&mut reader)?;
            let mut to_item_id = Vec::with_capacity(reference_count as usize);
            for _ in 0..reference_count {
                to_item_id.push(u16::deserialize(&mut reader)? as u32);
            }

            Ok(SingleItemTypeReferenceBox {
                header: seed.0,
                full_header: seed.1,
                from_item_id,
                reference_count,
                to_item_id,
            })
        } else {
            let from_item_id = u32::deserialize(&mut reader)?;
            let reference_count = u16::deserialize(&mut reader)?;
            let mut to_item_id = Vec::with_capacity(reference_count as usize);
            for _ in 0..reference_count {
                to_item_id.push(u32::deserialize(&mut reader)?);
            }

            Ok(SingleItemTypeReferenceBox {
                header: seed.0,
                full_header: seed.1,
                from_item_id,
                reference_count,
                to_item_id,
            })
        }
    }
}

impl Serialize for SingleItemTypeReferenceBox {
    fn serialize<W>(&self, mut writer: W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        if let Some(full_header) = &self.full_header {
            full_header.serialize(&mut writer)?;
            (self.from_item_id as u16).serialize(&mut writer)?;
            self.reference_count.serialize(&mut writer)?;

            for id in &self.to_item_id {
                (*id as u16).serialize(&mut writer)?;
            }

            Ok(())
        } else {
            self.header.serialize(&mut writer)?;
            self.from_item_id.serialize(&mut writer)?;
            self.reference_count.serialize(&mut writer)?;

            for id in &self.to_item_id {
                id.serialize(&mut writer)?;
            }

            Ok(())
        }
    }
}

impl IsoSized for SingleItemTypeReferenceBox {
    fn size(&self) -> usize {
        let mut size = 0;

        if let Some(full_header) = &self.full_header {
            size += full_header.size();
            size += 2; // from_item_id
            size += 2; // reference_count
            size += self.to_item_id.len() * 2; // to_item_id
        } else {
            size += self.header.size();
            size += 4; // from_item_id
            size += 2; // reference_count
            size += self.to_item_id.len() * 4; // to_item_id
        }

        size
    }
}

/// Item properties box
///
/// ISO/IEC 14496-12 - 8.11.14
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"iprp", crate_path = crate)]
pub struct ItemPropertiesBox<'a> {
    pub full_header: FullBoxHeader,
    #[iso_box(nested_box)]
    pub property_container: ItemPropertyContainerBox<'a>,
    #[iso_box(nested_box(collect))]
    pub association: Vec<ItemPropertyAssociationBox>,
}

#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"ipco", crate_path = crate)]
pub struct ItemPropertyContainerBox<'a> {
    #[iso_box(nested_box(collect))]
    pub etyp: Option<ExtendedTypeBox<'a>>,
    #[iso_box(nested_box(collect))]
    pub brnd: Vec<BrandProperty>,
    #[iso_box(nested_box(collect))]
    pub scrb: Vec<ScrambleSchemeInfoBox<'a>>,
    #[iso_box(nested_box(collect_unknown))]
    pub boxes: Vec<UnknownBox<'a>>,
}

#[derive(Debug, IsoBox)]
#[iso_box(box_type = b"ipma", skip_impl(deserialize_seed, serialize, sized), crate_path = crate)]
pub struct ItemPropertyAssociationBox {
    pub full_header: FullBoxHeader,
    pub entry_count: u32,
    pub entries: Vec<ItemPropertyAssociationBoxEntry>,
}

impl<'a> DeserializeSeed<'a, BoxHeader> for ItemPropertyAssociationBox {
    fn deserialize_seed<R>(mut reader: R, _seed: BoxHeader) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let full_header = FullBoxHeader::deserialize(&mut reader)?;
        let entry_count = u32::deserialize(&mut reader)?;

        let mut entries = Vec::with_capacity(entry_count as usize);
        for _ in 0..entry_count {
            entries.push(ItemPropertyAssociationBoxEntry::deserialize_seed(&mut reader, &full_header)?);
        }

        Ok(ItemPropertyAssociationBox {
            full_header,
            entry_count,
            entries,
        })
    }
}

impl Serialize for ItemPropertyAssociationBox {
    fn serialize<W>(&self, mut writer: W) -> io::Result<()>
    where
        W: std::io::Write,
    {
        self.serialize_box_header(&mut writer)?;
        self.full_header.serialize(&mut writer)?;
        self.entry_count.serialize(&mut writer)?;

        for entry in &self.entries {
            entry.serialize_seed(&mut writer, &self.full_header)?;
        }

        Ok(())
    }
}

impl IsoSized for ItemPropertyAssociationBox {
    fn size(&self) -> usize {
        let mut size = self.full_header.size();

        size += 4; // entry_count
        size += self.entries.iter().map(|e| e.size(&self.full_header)).sum::<usize>();

        Self::add_header_size(size)
    }
}

#[derive(Debug)]
pub struct ItemPropertyAssociationBoxEntry {
    pub item_id: u32,
    pub association_count: u8,
    pub associations: Vec<ItemPropertyAssociationBoxEntryAssociation>,
}

impl<'a> DeserializeSeed<'a, &FullBoxHeader> for ItemPropertyAssociationBoxEntry {
    fn deserialize_seed<R>(mut reader: R, seed: &FullBoxHeader) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let item_id = if seed.version < 1 {
            u16::deserialize(&mut reader)? as u32
        } else {
            u32::deserialize(&mut reader)?
        };

        let assocation_count = u8::deserialize(&mut reader)?;
        let mut associations = Vec::with_capacity(assocation_count as usize);
        for _ in 0..assocation_count {
            associations.push(ItemPropertyAssociationBoxEntryAssociation::deserialize_seed(
                &mut reader,
                seed,
            )?);
        }

        Ok(ItemPropertyAssociationBoxEntry {
            item_id,
            association_count: assocation_count,
            associations,
        })
    }
}

impl SerializeSeed<&FullBoxHeader> for ItemPropertyAssociationBoxEntry {
    fn serialize_seed<W>(&self, mut writer: W, seed: &FullBoxHeader) -> io::Result<()>
    where
        W: std::io::Write,
    {
        if seed.version < 1 {
            (self.item_id as u16).serialize(&mut writer)?;
        } else {
            self.item_id.serialize(&mut writer)?;
        }

        self.association_count.serialize(&mut writer)?;
        for association in &self.associations {
            association.serialize_seed(&mut writer, seed.flags)?;
        }

        Ok(())
    }
}

impl ItemPropertyAssociationBoxEntry {
    pub fn size(&self, header: &FullBoxHeader) -> usize {
        let mut size = 0;

        if header.version < 1 {
            size += 2; // item_id
        } else {
            size += 4; // item_id
        }
        size += 1; // association_count
        size += self.associations.iter().map(|a| a.size(header.flags)).sum::<usize>();

        size
    }
}

#[derive(Debug)]
pub struct ItemPropertyAssociationBoxEntryAssociation {
    pub essential: bool,
    pub property_index: u16,
}

impl<'a> DeserializeSeed<'a, &FullBoxHeader> for ItemPropertyAssociationBoxEntryAssociation {
    fn deserialize_seed<R>(mut reader: R, seed: &FullBoxHeader) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let byte = u8::deserialize(&mut reader)?;
        let essential = byte >> 7 != 0;
        let property_index = (byte & 0b0111_1111) as u16;

        if (*seed.flags & 0b1) != 0 {
            let low_byte = u8::deserialize(&mut reader)?;

            Ok(Self {
                essential,
                property_index: (property_index << 8) | low_byte as u16,
            })
        } else {
            Ok(Self {
                essential,
                property_index,
            })
        }
    }
}

impl SerializeSeed<U24Be> for ItemPropertyAssociationBoxEntryAssociation {
    fn serialize_seed<W>(&self, mut writer: W, seed: U24Be) -> io::Result<()>
    where
        W: std::io::Write,
    {
        // e iiiiiii
        let mut byte = (self.essential as u8) << 7;
        byte |= ((self.property_index >> 8) as u8) & 0b0111_1111;
        byte.serialize(&mut writer)?;

        if (*seed & 0b1) != 0 {
            // iiiiiiii
            let low_byte = (self.property_index & 0xFF) as u8;
            low_byte.serialize(&mut writer)?;
        }

        Ok(())
    }
}

impl ItemPropertyAssociationBoxEntryAssociation {
    pub fn size(&self, flags: U24Be) -> usize {
        if (*flags & 0b1) != 0 {
            2 // e iiiiiii + iiiiiiii
        } else {
            1 // e iiiiiii
        }
    }
}

/// Brand item property
///
/// ISO/IEC 14496-12 - 8.11.15
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"brnd", crate_path = crate)]
pub struct BrandProperty {
    #[iso_box(from = "[u8; 4]")]
    pub major_brand: Brand,
    pub minor_version: u32,
    #[iso_box(repeated, from = "[u8; 4]")]
    pub compatible_brands: Vec<Brand>,
}
