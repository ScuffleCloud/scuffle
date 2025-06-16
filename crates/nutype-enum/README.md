<!-- cargo-sync-rdme title [[ -->
# nutype-enum
<!-- cargo-sync-rdme ]] -->

> [!WARNING]  
> This crate is under active development and may not be stable.

<!-- cargo-sync-rdme badge [[ -->
![License: MIT OR Apache-2.0](https://img.shields.io/crates/l/nutype-enum.svg?style=flat-square)
[![docs.rs](https://img.shields.io/docsrs/nutype-enum.svg?logo=docs.rs&style=flat-square)](https://docs.rs/nutype-enum)
[![crates.io](https://img.shields.io/crates/v/nutype-enum.svg?logo=rust&style=flat-square)](https://crates.io/crates/nutype-enum)
[![GitHub Actions: ci](https://img.shields.io/github/actions/workflow/status/scufflecloud/scuffle/ci.yaml.svg?label=ci&logo=github&style=flat-square)](https://github.com/scufflecloud/scuffle/actions/workflows/ci.yaml)
[![Codecov](https://img.shields.io/codecov/c/github/scufflecloud/scuffle.svg?label=codecov&logo=codecov&style=flat-square)](https://codecov.io/gh/scufflecloud/scuffle)
<!-- cargo-sync-rdme ]] -->

---

<!-- cargo-sync-rdme rustdoc [[ -->
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
<!-- cargo-sync-rdme ]] -->
