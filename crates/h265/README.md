<!-- dprint-ignore-file -->
<!-- sync-readme title [[ -->
# scuffle-h265
<!-- sync-readme ]] -->

> [!WARNING]  
> This crate is under active development and may not be stable.

<!-- sync-readme badge [[ -->
[![docs.rs](https://img.shields.io/docsrs/scuffle-h265/0.2.2.svg?logo=docs.rs&label=docs.rs&style=flat-square)](https://docs.rs/scuffle-h265/0.2.2)
[![crates.io](https://img.shields.io/badge/crates.io-v0.2.2-orange?style=flat-square&logo=rust&logoColor=white)](https://crates.io/crates/scuffle-h265/0.2.2)
![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-purple.svg?style=flat-square)
![Crates.io Size](https://img.shields.io/crates/size/scuffle-h265/0.2.2.svg?style=flat-square)
![Crates.io Downloads](https://img.shields.io/crates/dv/scuffle-h265/0.2.2.svg?&label=downloads&style=flat-square)
[![Codecov](https://img.shields.io/codecov/c/github/scufflecloud/scuffle.svg?label=codecov&logo=codecov&style=flat-square)](https://app.codecov.io/gh/scufflecloud/scuffle)
<!-- sync-readme ]] -->

---

<!-- sync-readme rustdoc [[ -->
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
<!-- sync-readme ]] -->
