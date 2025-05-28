<!-- cargo-sync-rdme title [[ -->
# isobmff-derive
<!-- cargo-sync-rdme ]] -->

> [!WARNING]  
> This crate is under active development and may not be stable.

<!-- cargo-sync-rdme badge [[ -->
![License: MIT OR Apache-2.0](https://img.shields.io/crates/l/isobmff-derive.svg?style=flat-square)
[![docs.rs](https://img.shields.io/docsrs/isobmff-derive.svg?logo=docs.rs&style=flat-square)](https://docs.rs/isobmff-derive)
[![crates.io](https://img.shields.io/crates/v/isobmff-derive.svg?logo=rust&style=flat-square)](https://crates.io/crates/isobmff-derive)
[![GitHub Actions: ci](https://img.shields.io/github/actions/workflow/status/scufflecloud/scuffle/ci.yaml.svg?label=ci&logo=github&style=flat-square)](https://github.com/scufflecloud/scuffle/actions/workflows/ci.yaml)
[![Codecov](https://img.shields.io/codecov/c/github/scufflecloud/scuffle.svg?label=codecov&logo=codecov&style=flat-square)](https://codecov.io/gh/scufflecloud/scuffle)
<!-- cargo-sync-rdme ]] -->

---

<!-- cargo-sync-rdme rustdoc [[ -->
Derive helper macro for the `isobmff` crate.

Use this macro to implement the `IsoBox` trait as well as the resptive implementations of the
`Deserialize`, `DeserializeSeed`, `Serialize`, and `IsoSized` traits.

### Usage

This derive macro can only be used on structs with named fields.

All field types must implement the `Deserialize` and `Serialize` traits from the `scuffle_bytes_util` crate.
If that cannot be guaranteed, you should use the `from` field attribute. (See below)

#### Struct Attributes

|Attribute|Description|Required|
|---------|-----------|--------|
|`box_type`|The FourCC box type of the box. Provide as a byte array of size 4 (`[u8; 4]`).|Yes|
|`crate_path`|The path to the `isobmff` crate. Defaults to `::isobmff`.|No|
|`skip_impl`|A list of impls that should be skipped by the code generation. (i.e. you want to implement them manually). Defaults to none.|No|

#### Field Attributes

|Attribute|Description|Required|
|---------|-----------|--------|
|`from`|If specified, the provided type is parsed and then converted to the expected type using the [`From`](https://doc.rust-lang.org/nightly/core/convert/trait.From.html) trait.|No|
|`repeated`|Repeted fields are read repeatedly until the reader reaches EOF. There can only be one repeated field which should also appear as the last field in the struct.|No|
|`nested_box`|Can be used to make other boxes part of the box. The reader will read all boxes after the actual payload (the other fields) was read. Use `nested_box(collect)` to read optional/multiple boxes. Use `nested_box(collect_unknown)` to capture any unknown boxes.|No|

### Example

````rust
use isobmff::IsoBox;

#[derive(IsoBox)]
#[iso_box(box_type = b"myb1")]
pub struct MyCustomBox {
    pub foo: u32,
    pub bar: u8,
    #[iso_box(repeated)]
    pub baz: Vec<i16>,
}
````

The macro will generate code equivalent to this:

````rust
use isobmff::{BoxHeader, BoxType, IsoBox, IsoSized};
use scuffle_bytes_util::IoResultExt;
use scuffle_bytes_util::zero_copy::{Deserialize, DeserializeSeed, Serialize, ZeroCopyReader};

impl IsoBox for MyCustomBox {
    const TYPE: BoxType = BoxType::FourCc(*b"myb1");
}

impl<'a> Deserialize<'a> for MyCustomBox {
    fn deserialize<R>(mut reader: R) -> std::io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let seed = BoxHeader::deserialize(&mut reader)?;

        if let Some(size) = BoxHeader::payload_size(&seed) {
            Self::deserialize_seed(reader.take(size), seed)
        } else {
            Self::deserialize_seed(reader, seed)
        }
    }
}

impl<'a> DeserializeSeed<'a, BoxHeader> for MyCustomBox {
    fn deserialize_seed<R>(mut reader: R, seed: BoxHeader) -> std::io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let foo = u32::deserialize(&mut reader)?;
        let bar = u8::deserialize(&mut reader)?;

        let baz = {
            if let Some(payload_size) = seed.payload_size() {
                let mut payload_reader = reader.take(payload_size);
                std::iter::from_fn(|| {
                    i16::deserialize(&mut payload_reader).eof_to_none().transpose()
                }).collect::<Result<Vec<_>, std::io::Error>>()?
            } else {
                std::iter::from_fn(|| {
                    i16::deserialize(&mut reader).eof_to_none().transpose()
                }).collect::<Result<Vec<_>, std::io::Error>>()?
            }
        };

        Ok(Self { foo, bar, baz })
    }
}

impl Serialize for MyCustomBox {
    fn serialize<W>(&self, mut writer: W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        self.serialize_box_header(&mut writer)?;

        self.foo.serialize(&mut writer)?;
        self.bar.serialize(&mut writer)?;
        for item in &self.baz {
            item.serialize(&mut writer)?;
        }

        Ok(())
    }
}

impl IsoSized for MyCustomBox {
    fn size(&self) -> usize {
        Self::add_header_size(self.foo.size() + self.bar.size() + self.baz.size())
    }
}
````

### License

This project is licensed under the MIT or Apache-2.0 license.
You can choose between one of them if you use this work.

`SPDX-License-Identifier: MIT OR Apache-2.0`
<!-- cargo-sync-rdme ]] -->
