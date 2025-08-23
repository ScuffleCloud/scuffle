<!-- dprint-ignore-file -->
<!-- sync-readme title [[ -->
# nutype-enum
<!-- sync-readme ]] -->

> [!WARNING]  
> This crate is under active development and may not be stable.

<!-- sync-readme badge [[ -->
[![docs.rs](https://img.shields.io/docsrs/nutype-enum/0.1.5.svg?logo=docs.rs&label=docs.rs&style=flat-square)](https://docs.rs/nutype-enum/0.1.5)
[![crates.io](https://img.shields.io/badge/crates.io-v0.1.5-orange?style=flat-square&logo=rust&logoColor=white)](https://crates.io/crates/nutype-enum/0.1.5)
![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-purple.svg?style=flat-square)
![Crates.io Size](https://img.shields.io/crates/size/nutype-enum/0.1.5.svg?style=flat-square)
![Crates.io Downloads](https://img.shields.io/crates/dv/nutype-enum/0.1.5.svg?&label=downloads&style=flat-square)
[![Codecov](https://img.shields.io/codecov/c/github/scufflecloud/scuffle.svg?label=codecov&logo=codecov&style=flat-square)](https://app.codecov.io/gh/scufflecloud/scuffle)
<!-- sync-readme ]] -->

---

<!-- sync-readme rustdoc [[ -->
The crate provides a macro to create a new enum type with a single field.

See the [changelog](./CHANGELOG.md) for a full release history.

### Feature flags

* **`docs`** —  Enables changelog and documentation of feature flags

### Why do we need this?

This is useful when you have a value and you want to have enum like behavior and have a catch all case for all other values.

### Examples

````rust
use nutype_enum::nutype_enum;

nutype_enum! {
    pub enum AacPacketType(u8) {
        SeqHdr = 0x0,
        Raw = 0x1,
    }
}
````

### License

This project is licensed under the MIT or Apache-2.0 license.
You can choose between one of them if you use this work.

`SPDX-License-Identifier: MIT OR Apache-2.0`
<!-- sync-readme ]] -->
