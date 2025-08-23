<!-- dprint-ignore-file -->
<!-- sync-readme title [[ -->
# scuffle-flv
<!-- sync-readme ]] -->

> [!WARNING]  
> This crate is under active development and may not be stable.

<!-- sync-readme badge [[ -->
[![docs.rs](https://img.shields.io/docsrs/scuffle-flv/0.2.2.svg?logo=docs.rs&label=docs.rs&style=flat-square)](https://docs.rs/scuffle-flv/0.2.2)
[![crates.io](https://img.shields.io/badge/crates.io-v0.2.2-orange?style=flat-square&logo=rust&logoColor=white)](https://crates.io/crates/scuffle-flv/0.2.2)
![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-purple.svg?style=flat-square)
![Crates.io Size](https://img.shields.io/crates/size/scuffle-flv/0.2.2.svg?style=flat-square)
![Crates.io Downloads](https://img.shields.io/crates/dv/scuffle-flv/0.2.2.svg?&label=downloads&style=flat-square)
[![Codecov](https://img.shields.io/codecov/c/github/scufflecloud/scuffle.svg?label=codecov&logo=codecov&style=flat-square)](https://app.codecov.io/gh/scufflecloud/scuffle)
<!-- sync-readme ]] -->

---

<!-- sync-readme rustdoc [[ -->
A pure Rust implementation of the FLV format, allowing for demuxing of FLV
files and streams.

See the [changelog](./CHANGELOG.md) for a full release history.

### Feature flags

* **`docs`** —  Enables changelog and documentation of feature flags

### Specifications

|Name|Version|Link|Comments|
|----|-------|----|--------|
|Video File Format Specification|`10`|<https://github.com/veovera/enhanced-rtmp/blob/main/docs/legacy/video-file-format-v10-0-spec.pdf>||
|Adobe Flash Video File Format Specification|`10.1`|<https://github.com/veovera/enhanced-rtmp/blob/main/docs/legacy/video-file-format-v10-1-spec.pdf>|Refered to as ‘Legacy FLV spec’ in this documentation|
|Enhancing RTMP, FLV|`v1-2024-02-29-r1`|<https://github.com/veovera/enhanced-rtmp/blob/main/docs/enhanced/enhanced-rtmp-v1.pdf>||
|Enhanced RTMP|`v2-2024-10-22-b1`|<https://github.com/veovera/enhanced-rtmp/blob/main/docs/enhanced/enhanced-rtmp-v2.pdf>|Refered to as ‘Enhanced RTMP spec’ in this documentation|

### License

This project is licensed under the MIT or Apache-2.0 license.
You can choose between one of them if you use this work.

`SPDX-License-Identifier: MIT OR Apache-2.0`
<!-- sync-readme ]] -->
