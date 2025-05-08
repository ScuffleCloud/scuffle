use scuffle_bytes_util::zero_copy::{Deserialize, DeserializeSeed};
use scuffle_bytes_util::{BytesCow, IoResultExt};

use super::{Brand, DataInformationBox, HandlerBox, ProtectionSchemeInfoBox, ScrambleSchemeInfoBox};
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
#[derive(Debug)]
pub struct ItemLocationBox {
    pub header: FullBoxHeader,
    pub offset_size: u8,
    pub length_size: u8,
    pub base_offset_size: u8,
    pub index_size: Option<u8>,
    pub item_count: Option<u32>,
    pub items: Vec<ItemLocationBoxItem>,
}

impl IsoBox for ItemLocationBox {
    type Header = FullBoxHeader;

    const TYPE: [u8; 4] = *b"iloc";
}

impl<'a> Deserialize<'a> for ItemLocationBox {
    fn deserialize<R>(mut reader: R) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let header = BoxHeader::deserialize(&mut reader)?;
        let header = FullBoxHeader::deserialize_seed(&mut reader, header)?;
        ItemLocationBox::deserialize_seed(reader, header)
    }
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

        let index_size = if seed.version == 1 || seed.version == 2 {
            let value = byte & 0x0F;
            if ![0, 4, 8].contains(&value) {
                return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid index_size"));
            }
            Some(value)
        } else {
            None
        };

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
            let base_offset = pad_to_u64(reader.try_read(base_offset_size as usize)?);
            let extent_count = u16::deserialize(&mut reader)?;
            let mut extents = Vec::with_capacity(extent_count as usize);
            for _ in 0..extent_count {
                let item_reference_index = if let Some(index_size) = index_size {
                    if index_size > 0 {
                        Some(pad_to_u64(reader.try_read(index_size as usize)?))
                    } else {
                        None
                    }
                } else {
                    None
                };
                let extent_offset = pad_to_u64(reader.try_read(offset_size as usize)?);
                let extent_length = pad_to_u64(reader.try_read(length_size as usize)?);

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

fn pad_to_u64(bytes: BytesCow<'_>) -> u64 {
    // We copy the bytes into a 8 byte array and convert it to a u64
    assert!(bytes.len() <= 8);
    let mut buf = [0u8; 8];
    buf[4 - bytes.len()..].copy_from_slice(bytes.as_bytes());
    u64::from_be_bytes(buf)
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
#[derive(Debug)]
pub struct PrimaryItemBox {
    pub header: FullBoxHeader,
    pub item_id: u32,
}

impl IsoBox for PrimaryItemBox {
    type Header = FullBoxHeader;

    const TYPE: [u8; 4] = *b"pitm";
}

impl<'a> Deserialize<'a> for PrimaryItemBox {
    fn deserialize<R>(mut reader: R) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let header = BoxHeader::deserialize(&mut reader)?;
        let header = FullBoxHeader::deserialize_seed(&mut reader, header)?;
        PrimaryItemBox::deserialize_seed(reader, header)
    }
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
#[derive(Debug)]
pub struct ItemInfoBox<'a> {
    pub header: FullBoxHeader,
    pub entry_count: u32,
    pub item_infos: Vec<ItemInfoEntry<'a>>,
}

impl IsoBox for ItemInfoBox<'_> {
    type Header = FullBoxHeader;

    const TYPE: [u8; 4] = *b"iinf";
}

impl<'a> Deserialize<'a> for ItemInfoBox<'a> {
    fn deserialize<R>(mut reader: R) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let header = BoxHeader::deserialize(&mut reader)?;
        let header = FullBoxHeader::deserialize_seed(&mut reader, header)?;
        ItemInfoBox::deserialize_seed(reader, header)
    }
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

#[derive(Debug)]
pub struct ItemInfoEntry<'a> {
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
        content_encoding: Option<Utf8String>,
    },
    Uri {
        item_uri_type: Utf8String,
    },
}

impl IsoBox for ItemInfoEntry<'_> {
    type Header = FullBoxHeader;

    const TYPE: [u8; 4] = *b"infe";
}

impl<'a> Deserialize<'a> for ItemInfoEntry<'a> {
    fn deserialize<R>(mut reader: R) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let header = BoxHeader::deserialize(&mut reader)?;
        let header = FullBoxHeader::deserialize_seed(&mut reader, header)?;
        ItemInfoEntry::deserialize_seed(reader, header)
    }
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
                    content_encoding: if content_encoding.is_empty() {
                        None
                    } else {
                        Some(content_encoding)
                    },
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
#[derive(Debug)]
pub struct ItemReferenceBox {
    pub header: FullBoxHeader,
    pub references: Vec<SingleItemTypeReferenceBox>,
}

impl IsoBox for ItemReferenceBox {
    type Header = FullBoxHeader;

    const TYPE: [u8; 4] = *b"iref";
}

impl<'a> Deserialize<'a> for ItemReferenceBox {
    fn deserialize<R>(mut reader: R) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let header = BoxHeader::deserialize(&mut reader)?;
        let header = FullBoxHeader::deserialize_seed(&mut reader, header)?;
        Self::deserialize_seed(reader, header)
    }
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
            header: seed.header,
            from_item_id,
            reference_count,
            to_item_id,
        })
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

#[derive(Debug)]
pub struct ItemPropertyAssociationBox {
    pub header: FullBoxHeader,
    pub entry_count: u32,
    pub entries: Vec<ItemPropertyAssociationBoxEntry>,
}

impl IsoBox for ItemPropertyAssociationBox {
    type Header = FullBoxHeader;

    const TYPE: [u8; 4] = *b"ipma";
}

impl<'a> Deserialize<'a> for ItemPropertyAssociationBox {
    fn deserialize<R>(mut reader: R) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let header = BoxHeader::deserialize(&mut reader)?;
        let header = FullBoxHeader::deserialize_seed(&mut reader, header)?;
        ItemPropertyAssociationBox::deserialize_seed(reader, header)
    }
}

impl<'a> DeserializeSeed<'a, FullBoxHeader> for ItemPropertyAssociationBox {
    fn deserialize_seed<R>(mut reader: R, seed: FullBoxHeader) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let entry_count = u32::deserialize(&mut reader)?;

        let mut entries = Vec::with_capacity(entry_count as usize);
        for _ in 0..entry_count {
            let entry = ItemPropertyAssociationBoxEntry::deserialize_seed(&mut reader, &seed)?;
            entries.push(entry);
        }

        Ok(ItemPropertyAssociationBox {
            header: seed,
            entry_count,
            entries,
        })
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

        if seed.flags & 0b1 != 0 {
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
