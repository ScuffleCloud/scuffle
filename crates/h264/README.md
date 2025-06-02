<!-- dprint-ignore-file -->
<!-- sync-readme title [[ -->
# scuffle-h264
<!-- sync-readme ]] -->

> [!WARNING]  
> This crate is under active development and may not be stable.

<!-- sync-readme badge [[ -->
[![docs.rs](https://img.shields.io/docsrs/scuffle-h264/0.2.2.svg?logo=docs.rs&label=docs.rs&style=flat-square)](https://docs.rs/scuffle-h264/0.2.2)
[![crates.io](https://img.shields.io/badge/crates.io-v0.2.2-orange?style=flat-square&logo=rust&logoColor=white)](https://crates.io/crates/scuffle-h264/0.2.2)
![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-purple.svg?style=flat-square)
![Crates.io Size](https://img.shields.io/crates/size/scuffle-h264/0.2.2.svg?style=flat-square)
![Crates.io Downloads](https://img.shields.io/crates/dv/scuffle-h264/0.2.2.svg?&label=downloads&style=flat-square)
[![Codecov](https://img.shields.io/codecov/c/github/scufflecloud/scuffle.svg?label=codecov&logo=codecov&style=flat-square)](https://app.codecov.io/gh/scufflecloud/scuffle)
<!-- sync-readme ]] -->

---

<!-- sync-readme rustdoc [[ -->
A pure Rust implementation of the H.264 (header only) builder and parser.

This crate is designed to provide a simple and safe interface to build and parse H.264 headers.

See the [changelog](./CHANGELOG.md) for a full release history.

### Feature flags

* **`docs`** —  Enables changelog and documentation of feature flags
* **`isobmff`** —  Enables isobmff boxes

### Examples

#### Parsing

````rust
use std::io;

use scuffle_bytes_util::zero_copy::Deserialize;

use scuffle_h264::{AVCDecoderConfigurationRecord, Sps};

// A sample h264 bytestream to parse

// Parsing
let result = AVCDecoderConfigurationRecord::deserialize(scuffle_bytes_util::zero_copy::Slice::from(&bytes[..])).unwrap();

// Do something with it!

// You can also parse an Sps from the Sps struct:
let sps = Sps::parse_with_emulation_prevention(io::Cursor::new(&result.sps[0]));
````

For more examples, check out the tests in the source code for the parse function.

#### Building

````rust
use scuffle_bytes_util::BytesCow;
use scuffle_bytes_util::zero_copy::Serialize;

use scuffle_h264::{AVCDecoderConfigurationRecord, AvccExtendedConfig, Sps, SpsExtended};

let extended_config = AvccExtendedConfig {
    chroma_format_idc: 1,
    bit_depth_luma_minus8: 0,
    bit_depth_chroma_minus8: 0,
    sequence_parameter_set_ext: vec![SpsExtended {
        chroma_format_idc: 1,
        separate_color_plane_flag: false,
        bit_depth_luma_minus8: 2,
        bit_depth_chroma_minus8: 3,
        qpprime_y_zero_transform_bypass_flag: false,
        scaling_matrix: vec![],
    }],
};
let config = AVCDecoderConfigurationRecord {
    configuration_version: 1,
    profile_indication: 100,
    profile_compatibility: 0,
    level_indication: 31,
    length_size_minus_one: 3,
    sps: vec![
        BytesCow::from_static(b"spsdata"),
    ],
    pps: vec![BytesCow::from_static(b"ppsdata")],
    extended_config: Some(extended_config),
};

// Creating a buffer to store the built bytestream
let mut built = Vec::new();

// Building
config.serialize(&mut built).unwrap();

// Do something with it!
````

For more examples, check out the tests in the source code for the build function.

### License

This project is licensed under the MIT or Apache-2.0 license.
You can choose between one of them if you use this work.

`SPDX-License-Identifier: MIT OR Apache-2.0`
<!-- sync-readme ]] -->
