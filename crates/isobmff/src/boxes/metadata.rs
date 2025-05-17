use std::io;

use scuffle_bytes_util::zero_copy::{Deserialize, DeserializeSeed, Serialize};
use scuffle_bytes_util::{BitWriter, BytesCow, IoResultExt};

use super::{Brand, DataInformationBox, FDItemInformationBox, HandlerBox, ProtectionSchemeInfoBox, ScrambleSchemeInfoBox};
use crate::utils::pad_cow_to_u64;
use crate::{BoxHeader, FullBoxHeader, IsoBox, UnknownBox, Utf8String};

/// Meta box
///
/// ISO/IEC 14496-12 - 8.11.1
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"meta", crate_path = crate)]
pub struct MetaBox<'a> {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    #[iso_box(nested_box)]
    pub hdlr: HandlerBox,
    #[iso_box(nested_box(collect))]
    pub dinf: Option<DataInformationBox<'a>>,
    #[iso_box(nested_box(collect))]
    pub xml: Option<XmlBox>,
    #[iso_box(nested_box(collect))]
    pub bxml: Option<BinaryXmlBox<'a>>,
    #[iso_box(nested_box(collect))]
    pub iloc: Option<ItemLocationBox>,
    #[iso_box(nested_box(collect))]
    pub pitm: Option<PrimaryItemBox>,
    #[iso_box(nested_box(collect))]
    pub ipro: Option<ItemProtectionBox<'a>>,
    #[iso_box(nested_box(collect))]
    pub iinfo: Option<ItemInfoBox<'a>>,
    #[iso_box(nested_box(collect))]
    pub idat: Option<ItemDataBox<'a>>,
    #[iso_box(nested_box(collect))]
    pub iref: Option<ItemReferenceBox>,
    #[iso_box(nested_box(collect))]
    pub iprp: Option<ItemPropertiesBox<'a>>,
    #[iso_box(nested_box(collect))]
    pub fiin: Option<FDItemInformationBox>,
    #[iso_box(nested_box(collect_unknown))]
    pub unknown_boxes: Vec<UnknownBox<'a>>,
}

/// XML box
///
/// ISO/IEC 14496-12 - 8.11.2
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"xml ", crate_path = crate)]
pub struct XmlBox {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    pub xml: Utf8String,
}

/// Binary XML box
///
/// ISO/IEC 14496-12 - 8.11.2
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"bxml", crate_path = crate)]
pub struct BinaryXmlBox<'a> {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    pub data: BytesCow<'a>,
}

/// Item location box
///
/// ISO/IEC 14496-12 - 8.11.3
#[derive(Debug, IsoBox)]
#[iso_box(box_type = b"iloc", skip_impl(deserialize_seed, serialize), crate_path = crate)]
pub struct ItemLocationBox {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    pub offset_size: u8,
    pub length_size: u8,
    pub base_offset_size: u8,
    /// `index_size` or `reserved`
    pub index_size: u8,
    pub item_count: Option<u32>,
    pub items: Vec<ItemLocationBoxItem>,
}

impl<'a> DeserializeSeed<'a, FullBoxHeader> for ItemLocationBox {
    fn deserialize_seed<R>(mut reader: R, seed: FullBoxHeader) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
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
        if (seed.version == 1 || seed.version == 2) && ![0, 4, 8].contains(&index_size) {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid index_size"));
        }

        let item_count = if seed.version < 2 {
            Some(u16::deserialize(&mut reader)? as u32)
        } else if seed.version == 2 {
            Some(u32::deserialize(&mut reader)?)
        } else {
            None
        };

        let mut items = Vec::with_capacity(item_count.unwrap_or(0) as usize);
        for _ in 0..item_count.unwrap_or(0) {
            let item_id = if seed.version < 2 {
                Some(u16::deserialize(&mut reader)? as u32)
            } else if seed.version == 2 {
                Some(u32::deserialize(&mut reader)?)
            } else {
                None
            };

            let construction_method = if seed.version == 1 || seed.version == 2 {
                let value = u16::deserialize(&mut reader)?;
                Some((value & 0b1111) as u8)
            } else {
                None
            };

            let data_reference_index = u16::deserialize(&mut reader)?;
            let base_offset = pad_cow_to_u64(reader.try_read(base_offset_size as usize)?);
            let extent_count = u16::deserialize(&mut reader)?;
            let mut extents = Vec::with_capacity(extent_count as usize);
            for _ in 0..extent_count {
                let item_reference_index = if (seed.version == 1 || seed.version == 2) && index_size > 0 {
                    Some(pad_cow_to_u64(reader.try_read(index_size as usize)?))
                } else {
                    None
                };
                let extent_offset = pad_cow_to_u64(reader.try_read(offset_size as usize)?);
                let extent_length = pad_cow_to_u64(reader.try_read(length_size as usize)?);

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
            header: seed,
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

        self.header.serialize(&mut bit_writer)?;
        bit_writer.write_bits(self.offset_size as u64, 4)?;
        bit_writer.write_bits(self.length_size as u64, 4)?;
        bit_writer.write_bits(self.base_offset_size as u64, 4)?;
        bit_writer.write_bits(self.index_size as u64, 4)?;

        if self.header.version < 2 {
            (self
                .item_count
                .ok_or(io::Error::new(io::ErrorKind::InvalidData, "item_count is required"))? as u16)
                .serialize(&mut bit_writer)?;
        } else if self.header.version == 2 {
            self.item_count
                .ok_or(io::Error::new(io::ErrorKind::InvalidData, "item_count is required"))?
                .serialize(&mut bit_writer)?;
        }

        for item in &self.items {
            if self.header.version < 2 {
                (item
                    .item_id
                    .ok_or(io::Error::new(io::ErrorKind::InvalidData, "item_id is required"))? as u16)
                    .serialize(&mut bit_writer)?;
            } else if self.header.version == 2 {
                item.item_id
                    .ok_or(io::Error::new(io::ErrorKind::InvalidData, "item_id is required"))?
                    .serialize(&mut bit_writer)?;
            }

            if self.header.version == 1 || self.header.version == 2 {
                bit_writer.write_bits(0, 12)?;
                bit_writer.write_bits(
                    item.construction_method
                        .ok_or(io::Error::new(io::ErrorKind::InvalidData, "construction_method is required"))?
                        as u64,
                    4,
                )?;
            }

            item.data_reference_index.serialize(&mut bit_writer)?;
            bit_writer.write_bits(item.base_offset, self.base_offset_size * 8)?;
            item.extent_count.serialize(&mut bit_writer)?;

            for extent in &item.extents {
                if (self.header.version == 1 || self.header.version == 2) && self.index_size > 0 {
                    bit_writer.write_bits(
                        extent
                            .item_reference_index
                            .ok_or(io::Error::new(io::ErrorKind::InvalidData, "item_reference_index is required"))?,
                        self.index_size * 8,
                    )?;
                }
                bit_writer.write_bits(extent.extent_offset, self.offset_size * 8)?;
                bit_writer.write_bits(extent.extent_length, self.length_size * 8)?;
            }
        }

        Ok(())
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

#[derive(Debug)]
pub struct ItemLocationBoxExtent {
    pub item_reference_index: Option<u64>,
    pub extent_offset: u64,
    pub extent_length: u64,
}

/// Primary item box
///
/// ISO/IEC 14496-12 - 8.11.4
#[derive(Debug, IsoBox)]
#[iso_box(box_type = b"pitm", skip_impl(deserialize_seed), crate_path = crate)]
pub struct PrimaryItemBox {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    pub item_id: u32,
}

impl<'a> DeserializeSeed<'a, FullBoxHeader> for PrimaryItemBox {
    fn deserialize_seed<R>(mut reader: R, seed: FullBoxHeader) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let item_id = if seed.version == 0 {
            u16::deserialize(&mut reader)? as u32
        } else {
            u32::deserialize(&mut reader)?
        };

        Ok(PrimaryItemBox { header: seed, item_id })
    }
}

/// Item protection box
///
/// ISO/IEC 14496-12 - 8.11.5
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"ipro", crate_path = crate)]
pub struct ItemProtectionBox<'a> {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    pub protection_count: u16,
    #[iso_box(nested_box(collect))]
    pub protection_information: Vec<ProtectionSchemeInfoBox<'a>>,
}

/// Item information box
///
/// ISO/IEC 14496-12 - 8.11.6
#[derive(Debug, IsoBox)]
#[iso_box(box_type = b"iinf", skip_impl(deserialize_seed), crate_path = crate)]
pub struct ItemInfoBox<'a> {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    pub entry_count: u32,
    #[iso_box(nested_box(collect))]
    pub item_infos: Vec<ItemInfoEntry<'a>>,
}

impl<'a> DeserializeSeed<'a, FullBoxHeader> for ItemInfoBox<'a> {
    fn deserialize_seed<R>(mut reader: R, seed: FullBoxHeader) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let entry_count = if seed.version == 0 {
            u16::deserialize(&mut reader)? as u32
        } else {
            u32::deserialize(&mut reader)?
        };

        let mut item_infos = Vec::with_capacity(entry_count as usize);
        for _ in 0..entry_count {
            item_infos.push(ItemInfoEntry::deserialize(&mut reader)?);
        }

        Ok(ItemInfoBox {
            header: seed,
            entry_count,
            item_infos,
        })
    }
}

/// Item information entry
///
/// ISO/IEC 14496-12 - 8.11.6
#[derive(Debug, IsoBox)]
#[iso_box(box_type = b"infe", skip_impl(deserialize_seed, serialize), crate_path = crate)]
pub struct ItemInfoEntry<'a> {
    #[iso_box(header)]
    pub header: FullBoxHeader,
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

impl<'a> DeserializeSeed<'a, FullBoxHeader> for ItemInfoEntry<'a> {
    fn deserialize_seed<R>(mut reader: R, seed: FullBoxHeader) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let item_id = if seed.version == 0 || seed.version == 1 || seed.version == 2 {
            Some(u16::deserialize(&mut reader)? as u32)
        } else if seed.version == 3 {
            Some(u32::deserialize(&mut reader)?)
        } else {
            None
        };
        let item_protection_index = u16::deserialize(&mut reader)?;
        let item_type = if seed.version == 0 || seed.version == 1 {
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

        let extension_type = if seed.version == 1 {
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
            header: seed,
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
        self.header.serialize(&mut writer)?;

        if self.header.version == 0 || self.header.version == 1 {
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

        if self.header.version == 1 {
            if let Some(extension_type) = self.extension_type {
                extension_type.serialize(&mut writer)?;
            }
            if let Some(extension) = self.extension.as_ref() {
                extension.serialize(&mut writer)?;
            }
        }

        if self.header.version >= 2 {
            if self.header.version == 2 {
                (self
                    .item_id
                    .ok_or(io::Error::new(io::ErrorKind::InvalidData, "item_ID is required"))? as u16)
                    .serialize(&mut writer)?;
            } else if self.header.version == 3 {
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

// 8.11.7 is deprecated

// 8.11.8 is deprecated

/// Item data box
///
/// ISO/IEC 14496-12 - 8.11.11
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"idat", crate_path = crate)]
pub struct ItemDataBox<'a> {
    #[iso_box(header)]
    pub header: BoxHeader,
    pub data: BytesCow<'a>,
}

/// Item reference box
///
/// ISO/IEC 14496-12 - 8.11.12
#[derive(Debug, IsoBox)]
#[iso_box(box_type = b"iref", skip_impl(deserialize_seed), crate_path = crate)]
pub struct ItemReferenceBox {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    #[iso_box(repeated)]
    pub references: Vec<SingleItemTypeReferenceBox>,
}

impl<'a> DeserializeSeed<'a, FullBoxHeader> for ItemReferenceBox {
    fn deserialize_seed<R>(mut reader: R, seed: FullBoxHeader) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let mut references = Vec::new();

        loop {
            let Some(header) = BoxHeader::deserialize(&mut reader).eof_to_none()? else {
                break;
            };

            if seed.version == 0 {
                let Some(iso_box) = SingleItemTypeReferenceBox::deserialize_seed(&mut reader, header).eof_to_none()? else {
                    break;
                };
                references.push(iso_box);
            } else if seed.version == 1 {
                let Some(header) = FullBoxHeader::deserialize_seed(&mut reader, header).eof_to_none()? else {
                    break;
                };
                let Some(iso_box) = SingleItemTypeReferenceBox::deserialize_seed(&mut reader, header).eof_to_none()? else {
                    break;
                };
                references.push(iso_box);
            }
        }

        Ok(ItemReferenceBox {
            header: seed,
            references,
        })
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

impl<'a> DeserializeSeed<'a, BoxHeader> for SingleItemTypeReferenceBox {
    fn deserialize_seed<R>(mut reader: R, seed: BoxHeader) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let from_item_id = u16::deserialize(&mut reader)? as u32;
        let reference_count = u16::deserialize(&mut reader)?;
        let mut to_item_id = Vec::with_capacity(reference_count as usize);
        for _ in 0..reference_count {
            to_item_id.push(u16::deserialize(&mut reader)? as u32);
        }

        Ok(SingleItemTypeReferenceBox {
            header: seed,
            full_header: None,
            from_item_id,
            reference_count,
            to_item_id,
        })
    }
}

impl<'a> DeserializeSeed<'a, FullBoxHeader> for SingleItemTypeReferenceBox {
    fn deserialize_seed<R>(mut reader: R, seed: FullBoxHeader) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let from_item_id = u32::deserialize(&mut reader)?;
        let reference_count = u16::deserialize(&mut reader)?;
        let mut to_item_id = Vec::with_capacity(reference_count as usize);
        for _ in 0..reference_count {
            to_item_id.push(u32::deserialize(&mut reader)?);
        }

        Ok(SingleItemTypeReferenceBox {
            header: seed.header.clone(),
            full_header: Some(seed),
            from_item_id,
            reference_count,
            to_item_id,
        })
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

/// Item properties box
///
/// ISO/IEC 14496-12 - 8.11.14
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"iprp", crate_path = crate)]
pub struct ItemPropertiesBox<'a> {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    #[iso_box(nested_box)]
    pub property_container: ItemPropertyContainerBox<'a>,
    #[iso_box(nested_box(collect))]
    pub association: Vec<ItemPropertyAssociationBox>,
}

#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"ipco", crate_path = crate)]
pub struct ItemPropertyContainerBox<'a> {
    #[iso_box(header)]
    pub header: BoxHeader,
    #[iso_box(nested_box(collect))]
    pub brnd: Vec<BrandProperty>,
    #[iso_box(nested_box(collect))]
    pub scrb: Vec<ScrambleSchemeInfoBox<'a>>,
    #[iso_box(nested_box(collect_unknown))]
    pub boxes: Vec<UnknownBox<'a>>,
}

#[derive(Debug, IsoBox)]
#[iso_box(box_type = b"ipma", skip_impl(deserialize_seed, serialize), crate_path = crate)]
pub struct ItemPropertyAssociationBox {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    pub entry_count: u32,
    pub entries: Vec<ItemPropertyAssociationBoxEntry>,
}

impl<'a> DeserializeSeed<'a, FullBoxHeader> for ItemPropertyAssociationBox {
    fn deserialize_seed<R>(mut reader: R, seed: FullBoxHeader) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let entry_count = u32::deserialize(&mut reader)?;

        let mut entries = Vec::with_capacity(entry_count as usize);
        for _ in 0..entry_count {
            entries.push(ItemPropertyAssociationBoxEntry::deserialize_seed(&mut reader, &seed)?);
        }

        Ok(ItemPropertyAssociationBox {
            header: seed,
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
        self.header.serialize(&mut writer)?;
        self.entry_count.serialize(&mut writer)?;

        for entry in &self.entries {
            entry.serialize(&mut writer, &self.header)?;
        }

        Ok(())
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
            let association = ItemPropertyAssociationBoxEntryAssociation::deserialize_seed(&mut reader, seed)?;
            associations.push(association);
        }

        Ok(ItemPropertyAssociationBoxEntry {
            item_id,
            association_count: assocation_count,
            associations,
        })
    }
}

impl ItemPropertyAssociationBoxEntry {
    pub fn serialize<W>(&self, mut writer: W, header: &FullBoxHeader) -> io::Result<()>
    where
        W: std::io::Write,
    {
        if header.version < 1 {
            (self.item_id as u16).serialize(&mut writer)?;
        } else {
            self.item_id.serialize(&mut writer)?;
        }

        self.association_count.serialize(&mut writer)?;
        for association in &self.associations {
            association.serialize(&mut writer, header)?;
        }

        Ok(())
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

impl ItemPropertyAssociationBoxEntryAssociation {
    pub fn serialize<W>(&self, mut writer: W, header: &FullBoxHeader) -> io::Result<()>
    where
        W: std::io::Write,
    {
        // e iiiiiii
        let mut byte = (self.essential as u8) << 7;
        byte |= ((self.property_index >> 8) as u8) & 0b0111_1111;
        byte.serialize(&mut writer)?;

        if (*header.flags & 0b1) != 0 {
            // iiiiiiii
            let low_byte = (self.property_index & 0xFF) as u8;
            low_byte.serialize(&mut writer)?;
        }

        Ok(())
    }
}

/// Brand item property
///
/// ISO/IEC 14496-12 - 8.11.15
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"brnd", crate_path = crate)]
pub struct BrandProperty {
    #[iso_box(header)]
    pub header: BoxHeader,
    #[iso_box(from = "[u8; 4]")]
    pub major_brand: Brand,
    pub minor_version: u32,
    #[iso_box(repeated, from = "[u8; 4]")]
    pub compatible_brands: Vec<Brand>,
}
