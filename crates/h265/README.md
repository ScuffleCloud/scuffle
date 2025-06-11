<!-- cargo-sync-rdme title [[ -->
# scuffle-h265
<!-- cargo-sync-rdme ]] -->

> [!WARNING]  
> This crate is under active development and may not be stable.

<!-- cargo-sync-rdme badge [[ -->
![License: MIT OR Apache-2.0](https://img.shields.io/crates/l/scuffle-h265.svg?style=flat-square)
[![docs.rs](https://img.shields.io/docsrs/scuffle-h265.svg?logo=docs.rs&style=flat-square)](https://docs.rs/scuffle-h265)
[![crates.io](https://img.shields.io/crates/v/scuffle-h265.svg?logo=rust&style=flat-square)](https://crates.io/crates/scuffle-h265)
[![GitHub Actions: ci](https://img.shields.io/github/actions/workflow/status/scufflecloud/scuffle/ci.yaml.svg?label=ci&logo=github&style=flat-square)](https://github.com/scufflecloud/scuffle/actions/workflows/ci.yaml)
[![Codecov](https://img.shields.io/codecov/c/github/scufflecloud/scuffle.svg?label=codecov&logo=codecov&style=flat-square)](https://codecov.io/gh/scufflecloud/scuffle)
<!-- cargo-sync-rdme ]] -->

---

<!-- cargo-sync-rdme rustdoc [[ -->
A pure Rust implementation of the HEVC/H.265 decoder.

This crate is designed to provide a simple and safe interface to decode HEVC/H.265 SPS NALUs.

See the [changelog](./CHANGELOG.md) for a full release history.

### Feature flags

* **`docs`** —  Enables changelog and documentation of feature flags
* **`isobmff`** —  Enables isobmff boxes

### Examples

````rust
use scuffle_h265::SpsNALUnit;

let nalu = SpsNALUnit::parse(reader)?;
println!("Parsed SPS NALU: {:?}", nalu);
````

### License

This project is licensed under the MIT or Apache-2.0 license.
You can choose between one of them if you use this work.

`SPDX-License-Identifier: MIT OR Apache-2.0`
<!-- cargo-sync-rdme ]] -->
