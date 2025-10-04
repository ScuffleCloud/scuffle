use std::fmt::Debug;
use std::io;

use scuffle_bytes_util::zero_copy::{Deserialize, DeserializeSeed, Serialize, U24Be};
use scuffle_bytes_util::{BitWriter, BytesCow, IoResultExt};

use super::{
    Brand, DataInformationBox, ExtendedTypeBox, FDItemInformationBox, GroupsListBox, HandlerBox, ProtectionSchemeInfoBox,
    ScrambleSchemeInfoBox,
};
use crate::{BoxHeader, FullBoxHeader, IsoBox, IsoSized, UnknownBox, Utf8String};

/// Meta box
///
/// ISO/IEC 14496-12 - 8.11.1
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"meta", crate_path = crate)]
pub struct MetaBox<'a> {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// The contained [`HandlerBox`]. (mandatory)
    #[iso_box(nested_box)]
    pub hdlr: HandlerBox,
    /// The contained [`DataInformationBox`]. (optional)
    #[iso_box(nested_box(collect))]
    pub dinf: Option<DataInformationBox<'a>>,
    /// The contained [`ItemLocationBox`]. (optional)
    #[iso_box(nested_box(collect))]
    pub iloc: Option<ItemLocationBox>,
    /// The contained [`ItemProtectionBox`]. (optional)
    #[iso_box(nested_box(collect))]
    pub ipro: Option<ItemProtectionBox<'a>>,
    /// The contained [`ItemInfoBox`]. (optional)
    #[iso_box(nested_box(collect))]
    pub iinf: Option<ItemInfoBox<'a>>,
    /// The contained [`XmlBox`]. (optional)
    #[iso_box(nested_box(collect))]
    pub xml: Option<XmlBox>,
    /// The contained [`BinaryXmlBox`]. (optional)
    #[iso_box(nested_box(collect))]
    pub bxml: Option<BinaryXmlBox<'a>>,
    /// The contained [`PrimaryItemBox`]. (optional)
    #[iso_box(nested_box(collect))]
    pub pitm: Option<PrimaryItemBox>,
    /// The contained [`FDItemInformationBox`]. (optional)
    #[iso_box(nested_box(collect))]
    pub fiin: Option<FDItemInformationBox>,
    /// The contained [`ItemDataBox`]. (optional)
    #[iso_box(nested_box(collect))]
    pub idat: Option<ItemDataBox<'a>>,
    /// The contained [`ItemReferenceBox`]. (optional)
    #[iso_box(nested_box(collect))]
    pub iref: Option<ItemReferenceBox>,
    /// The contained [`ItemPropertiesBox`]. (optional)
    #[iso_box(nested_box(collect))]
    pub iprp: Option<ItemPropertiesBox<'a>>,
    /// The contained [`GroupsListBox`]. (optional)
    #[iso_box(nested_box(collect))]
    pub grpl: Option<GroupsListBox<'a>>,
    /// A list of unknown boxes that were not recognized during deserialization.
    #[iso_box(nested_box(collect_unknown))]
    pub unknown_boxes: Vec<UnknownBox<'a>>,
}

/// XML box
///
/// ISO/IEC 14496-12 - 8.11.2
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"xml ", crate_path = crate)]
pub struct XmlBox {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// A string containing the XML data.
    pub xml: Utf8String,
}

/// Binary XML box
///
/// ISO/IEC 14496-12 - 8.11.2
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"bxml", crate_path = crate)]
pub struct BinaryXmlBox<'a> {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// Contains the encoded XML data.
    pub data: BytesCow<'a>,
}

/// Item location box
///
/// ISO/IEC 14496-12 - 8.11.3
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"iloc", skip_impl(deserialize_seed, serialize, sized), crate_path = crate)]
pub struct ItemLocationBox {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// Taken from the set {0, 4, 8} and indicates the length in bytes of the `offset` field.
    pub offset_size: u8,
    /// Taken from the set {0, 4, 8} and indicates the length in bytes of the `length` field.
    pub length_size: u8,
    /// Taken from the set {0, 4, 8} and indicates the length in bytes of the `base_offset` field.
    pub base_offset_size: u8,
    /// Taken from the set {0, 4, 8} and indicates the length in bytes of the `item_reference_index` field.
    ///
    /// If version is not 1 or 2, this field is reserved and does not represent the `index_size`.
    pub index_size: u8,
    /// Counts the number of resources in the [`items`](Self::items) array.
    pub item_count: Option<u32>,
    /// The items contained in this box.
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
            item.serialize(&mut bit_writer, self)?;
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

/// Item in the [`ItemLocationBox`].
#[derive(Debug, PartialEq, Eq)]
pub struct ItemLocationBoxItem {
    /// An arbitrary integer 'name' for this resource which can be used to refer to it (e.g. in a URL).
    pub item_id: Option<u32>,
    /// Taken from the set 0 (file), 1 (idat) or 2 (item).
    pub construction_method: Option<u8>,
    /// Either zero ('this file') or an index, with value 1 indicating the first entry, into
    /// the data references in the [`DataInformationBox`].
    pub data_reference_index: u16,
    /// Provides a base value for offset calculations within the referenced data.
    /// If `base_offset_size` is 0, `base_offset` takes the value 0, i.e. it is unused.
    pub base_offset: u64,
    /// Provides the count of the number of extents into which the resource is fragmented;
    /// it shall have the value 1 or greater.
    pub extent_count: u16,
    /// Extents in this item.
    pub extents: Vec<ItemLocationBoxExtent>,
}

impl ItemLocationBoxItem {
    fn serialize<W>(&self, writer: W, parent: &ItemLocationBox) -> io::Result<()>
    where
        W: std::io::Write,
    {
        let mut bit_writer = BitWriter::new(writer);

        if parent.full_header.version < 2 {
            (self
                .item_id
                .ok_or(io::Error::new(io::ErrorKind::InvalidData, "item_id is required"))? as u16)
                .serialize(&mut bit_writer)?;
        } else if parent.full_header.version == 2 {
            self.item_id
                .ok_or(io::Error::new(io::ErrorKind::InvalidData, "item_id is required"))?
                .serialize(&mut bit_writer)?;
        }

        if parent.full_header.version == 1 || parent.full_header.version == 2 {
            bit_writer.write_bits(0, 12)?;
            bit_writer.write_bits(
                self.construction_method
                    .ok_or(io::Error::new(io::ErrorKind::InvalidData, "construction_method is required"))?
                    as u64,
                4,
            )?;
        }

        self.data_reference_index.serialize(&mut bit_writer)?;
        bit_writer.write_bits(self.base_offset, parent.base_offset_size * 8)?;
        self.extent_count.serialize(&mut bit_writer)?;

        for extent in &self.extents {
            extent.serialize(&mut bit_writer, parent)?;
        }

        Ok(())
    }
}

impl ItemLocationBoxItem {
    /// Calculates the size of this item, depending on the parent box.
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

/// Extent in the [`ItemLocationBoxItem`].
#[derive(Debug, PartialEq, Eq)]
pub struct ItemLocationBoxExtent {
    /// Provides an index as defined for the construction method.
    pub item_reference_index: Option<u64>,
    /// Provides the absolute offset, in bytes from the data origin of the container, of this extent
    /// data. If [`offset_size`](ItemLocationBox::offset_size) is 0, `extent_offset` takes the value 0.
    pub extent_offset: u64,
    /// Provides the absolute length in bytes of this metadata item extent.
    /// If [`length_size`](ItemLocationBox::length_size) is 0, `extent_length` takes the value 0.
    /// If the value is 0, then length of the extent is the length of the entire referenced container.
    pub extent_length: u64,
}

impl ItemLocationBoxExtent {
    fn serialize<W>(&self, writer: W, parent: &ItemLocationBox) -> io::Result<()>
    where
        W: std::io::Write,
    {
        let mut bit_writer = BitWriter::new(writer);

        if (parent.full_header.version == 1 || parent.full_header.version == 2) && parent.index_size > 0 {
            bit_writer.write_bits(
                self.item_reference_index
                    .ok_or(io::Error::new(io::ErrorKind::InvalidData, "item_reference_index is required"))?,
                parent.index_size * 8,
            )?;
        }
        bit_writer.write_bits(self.extent_offset, parent.offset_size * 8)?;
        bit_writer.write_bits(self.extent_length, parent.length_size * 8)?;

        Ok(())
    }
}

impl ItemLocationBoxExtent {
    /// Calculates the size of this extent, depending on the parent box.
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
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"pitm", skip_impl(deserialize_seed, serialize, sized), crate_path = crate)]
pub struct PrimaryItemBox {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// The identifier of the primary item, which shall be the identifier of an item in the [`MetaBox`]
    /// containing the [`PrimaryItemBox`]. Version 1 should only be used when large `item_ID` values (exceeding
    /// 65535) are required or expected to be required.
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
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"ipro", crate_path = crate)]
pub struct ItemProtectionBox<'a> {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// Number of [`ProtectionSchemeInfoBox`]es in this box.
    pub protection_count: u16,
    /// The contained [`ScrambleSchemeInfoBox`]es. (one or more)
    #[iso_box(nested_box(collect))]
    pub protection_information: Vec<ProtectionSchemeInfoBox<'a>>,
}

/// Item information box
///
/// ISO/IEC 14496-12 - 8.11.6
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"iinf", skip_impl(deserialize_seed, serialize, sized), crate_path = crate)]
pub struct ItemInfoBox<'a> {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// Provides a count of the number of entries in the [`item_infos`](Self::item_infos) vec.
    pub entry_count: u32,
    /// The entries.
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
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"infe", skip_impl(deserialize_seed, serialize, sized), crate_path = crate)]
pub struct ItemInfoEntry<'a> {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// Contains either 0 for the primary resource (e.g., the XML contained in an XMLBox) or the ID of
    /// the item for which the following information is defined.
    pub item_id: Option<u32>,
    /// Contains either 0 for an unprotected item, or the index, with value 1 indicating
    /// the first entry, into the ItemProtectionBox defining the protection applied to this item (the first
    /// box in the ItemProtectionBox has the index 1).
    pub item_protection_index: u16,
    /// A 32-bit value, typically 4 printable characters, that is a defined valid item type indicator,
    /// such as 'mime'.
    pub item_type: [u8; 4],
    /// The symbolic name of the item (source file for file delivery transmissions).
    pub item_name: Utf8String,
    /// The item information.
    pub item: Option<ItemInfoEntryItem>,
    /// A four character code that identifies the extension fields.
    pub extension_type: Option<[u8; 4]>,
    /// The extension.
    pub extension: Option<ItemInfoExtension<'a>>,
}

/// Info in [`ItemInfoEntry`].
#[derive(Debug, PartialEq, Eq)]
pub enum ItemInfoEntryItem {
    /// MIME type item
    Mime {
        /// The MIME type of the item. If the item is content encoded, then the content
        /// type refers to the item after content decoding.
        content_type: Utf8String,
        /// Indicates that the binary file is encoded and needs to be decoded before
        /// interpreted. The values are as defined for `Content-Encoding` for HTTP/1.1. Some possible values are
        /// "gzip", "compress" and "deflate". An empty string indicates no content encoding. Note that the item
        /// is stored after the content encoding has been applied.
        content_encoding: Utf8String,
    },
    /// URI item
    Uri {
        /// An absolute URI, that is used as a type indicator.
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
            match &self.item {
                Some(ItemInfoEntryItem::Mime {
                    content_type,
                    content_encoding,
                }) => {
                    content_type.serialize(&mut writer)?;
                    content_encoding.serialize(&mut writer)?;
                }
                Some(ItemInfoEntryItem::Uri { item_uri_type }) => {
                    item_uri_type.serialize(&mut writer)?;
                }
                None => {}
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

/// [`ItemInfoEntry`] extension.
#[derive(Debug, PartialEq, Eq)]
pub enum ItemInfoExtension<'a> {
    /// "fdel"
    FDItemInfoExtension {
        /// Contains the URI of the file as defined in HTTP/1.1 (IETF RFC 2616).
        current_location: Utf8String,
        /// Contains an MD5 digest of the file.
        /// See HTTP/1.1 (IETF RFC 2616) and IETF RFC 1864.
        current_md5: Utf8String,
        /// Gives the total length (in bytes) of the (un-encoded) file.
        content_length: u64,
        /// Gives the total length (in bytes) of the (encoded) file. Transfer length is equal to
        /// content length if no content encoding is applied (see above).
        transfer_length: u64,
        /// Provides a count of the number of entries in the
        /// [`group_id`](ItemInfoExtension::FDItemInfoExtension::entry_count) vec.
        entry_count: u8,
        /// Indicates a file group to which the file item (source file) belongs. See 3GPP TS 26.346 for
        /// more details on file groups.
        group_id: Vec<u32>,
    },
    /// Any other extension.
    Other {
        /// The four character code that identifies the extension fields.
        extension_type: [u8; 4],
        /// Extension fields.
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
#[derive(IsoBox, PartialEq, Eq)]
#[iso_box(box_type = b"idat", crate_path = crate)]
pub struct ItemDataBox<'a> {
    /// The contained metadata.
    pub data: BytesCow<'a>,
}

impl Debug for ItemDataBox<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ItemDataBox").field("data.len", &self.data.len()).finish()
    }
}

/// Item reference box
///
/// ISO/IEC 14496-12 - 8.11.12
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"iref", skip_impl(deserialize_seed), crate_path = crate)]
pub struct ItemReferenceBox {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// The contained [`SingleItemTypeReferenceBox`]es. (any quantity)
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

            let Some(iso_box) =
                SingleItemTypeReferenceBox::deserialize_seed(&mut reader, (header, full_header.version == 1))
                    .eof_to_none()?
            else {
                break;
            };
            references.push(iso_box);
        }

        Ok(ItemReferenceBox { full_header, references })
    }
}

/// Single item type reference box
///
/// ISO/IEC 14496-12 - 8.11.12
#[derive(Debug, PartialEq, Eq)]
pub struct SingleItemTypeReferenceBox {
    /// Inidicates whether this is a `SingleItemTypeReferenceBox` or a `SingleItemTypeReferenceBoxLarge`.
    pub large: bool,
    /// The box header.
    pub header: BoxHeader,
    /// The `item_ID` of the item that refers to other items.
    pub from_item_id: u32,
    /// The number of references.
    pub reference_count: u16,
    /// The `item_ID` of the item referred to.
    pub to_item_id: Vec<u32>,
}

impl<'a> DeserializeSeed<'a, (BoxHeader, bool)> for SingleItemTypeReferenceBox {
    fn deserialize_seed<R>(mut reader: R, seed: (BoxHeader, bool)) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let (header, large) = seed;

        let from_item_id = if !large {
            u16::deserialize(&mut reader)? as u32
        } else {
            u32::deserialize(&mut reader)?
        };
        let reference_count = u16::deserialize(&mut reader)?;
        let mut to_item_id = Vec::with_capacity(reference_count as usize);
        for _ in 0..reference_count {
            if !large {
                to_item_id.push(u16::deserialize(&mut reader)? as u32);
            } else {
                to_item_id.push(u32::deserialize(&mut reader)?);
            }
        }

        Ok(SingleItemTypeReferenceBox {
            large,
            header,
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
        self.header.serialize(&mut writer)?;

        if !self.large {
            (self.from_item_id as u16).serialize(&mut writer)?;
        } else {
            self.from_item_id.serialize(&mut writer)?;
        }
        self.reference_count.serialize(&mut writer)?;
        for id in &self.to_item_id {
            if !self.large {
                (*id as u16).serialize(&mut writer)?;
            } else {
                id.serialize(&mut writer)?;
            }
        }

        Ok(())
    }
}

impl IsoSized for SingleItemTypeReferenceBox {
    fn size(&self) -> usize {
        let mut size = self.header.size();

        if !self.large {
            size += 2; // from_item_id
            size += 2; // reference_count
            size += self.to_item_id.len() * 2; // to_item_id
        } else {
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
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"iprp", crate_path = crate)]
pub struct ItemPropertiesBox<'a> {
    /// The contained [`ItemPropertyContainerBox`]. (mandatory)
    #[iso_box(nested_box)]
    pub property_container: ItemPropertyContainerBox<'a>,
    /// The contained [`ItemPropertyAssociationBox`]es. (any quantity)
    #[iso_box(nested_box(collect))]
    pub association: Vec<ItemPropertyAssociationBox>,
}

/// Item property container box
///
/// ISO/IEC 14496-12 - 8.11.14
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"ipco", crate_path = crate)]
pub struct ItemPropertyContainerBox<'a> {
    /// The contained [`ExtendedTypeBox`]. (optional)
    #[iso_box(nested_box(collect))]
    pub etyp: Option<ExtendedTypeBox<'a>>,
    /// The contained [`BrandProperty`]es. (zero or one per item)
    #[iso_box(nested_box(collect))]
    pub brnd: Vec<BrandProperty>,
    /// The contained [`ScrambleSchemeInfoBox`]es. (one or more)
    #[iso_box(nested_box(collect))]
    pub scrb: Vec<ScrambleSchemeInfoBox<'a>>,
    /// Any other sub boxes.
    #[iso_box(nested_box(collect_unknown))]
    pub boxes: Vec<UnknownBox<'a>>,
}

/// Item property association box
///
/// ISO/IEC 14496-12 - 8.11.14
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"ipma", skip_impl(deserialize_seed, serialize, sized), crate_path = crate)]
pub struct ItemPropertyAssociationBox {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// The number of entries in the [`entries`](Self::entries) vec.
    pub entry_count: u32,
    /// The contained entries.
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
            entry.serialize(&mut writer, &self.full_header)?;
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

/// Entry in the [`ItemPropertyAssociationBox`].
#[derive(Debug, PartialEq, Eq)]
pub struct ItemPropertyAssociationBoxEntry {
    /// Identifies the item with which properties are associated.
    pub item_id: u32,
    /// The number of associations in the [`associations`](Self::associations) vec.
    pub association_count: u8,
    /// The associations.
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

impl ItemPropertyAssociationBoxEntry {
    fn serialize<W>(&self, mut writer: W, header: &FullBoxHeader) -> io::Result<()>
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
            association.serialize(&mut writer, header.flags)?;
        }

        Ok(())
    }
}

impl ItemPropertyAssociationBoxEntry {
    /// Calculates the size of this entry, depending on the parent's box header.
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

/// Association in the [`ItemPropertyAssociationBoxEntry`].
#[derive(Debug, PartialEq, Eq)]
pub struct ItemPropertyAssociationBoxEntryAssociation {
    /// When set to 1 indicates that the associated property is essential to the item, otherwise it is non-essential.
    pub essential: bool,
    /// Either 0 indicating that no property is associated (the essential indicator shall also
    /// be 0), or is the 1-based index (counting all boxes, including [`FreeSpaceBox`](super::FreeSpaceBox)es)
    /// of the associated property box in the [`ItemPropertyContainerBox`] contained in the same [`ItemPropertiesBox`].
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
    fn serialize<W>(&self, mut writer: W, flags: U24Be) -> io::Result<()>
    where
        W: std::io::Write,
    {
        if (*flags & 0b1) != 0 {
            // e iiiiiii
            let mut byte = (self.essential as u8) << 7;
            byte |= ((self.property_index >> 8) as u8) & 0b0111_1111;
            byte.serialize(&mut writer)?;

            // iiiiiiii
            let low_byte = (self.property_index & 0xFF) as u8;
            low_byte.serialize(&mut writer)?;
        } else {
            // e iiiiiii
            let byte = (self.essential as u8) << 7 | (self.property_index & 0b0111_1111) as u8;
            byte.serialize(&mut writer)?;
        }

        Ok(())
    }
}

impl ItemPropertyAssociationBoxEntryAssociation {
    /// Calculates the size of this association, depending on the flags.
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
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"brnd", crate_path = crate)]
pub struct BrandProperty {
    /// The "best use" brand of the file which will provide the greatest compatibility.
    #[iso_box(from = "[u8; 4]")]
    pub major_brand: Brand,
    /// Minor version of the major brand.
    pub minor_version: u32,
    /// A list of compatible brands.
    #[iso_box(repeated, from = "[u8; 4]")]
    pub compatible_brands: Vec<Brand>,
}
