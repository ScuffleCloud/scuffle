<!-- cargo-sync-rdme title [[ -->
# scuffle-flv
<!-- cargo-sync-rdme ]] -->

> [!WARNING]  
> This crate is under active development and may not be stable.

<!-- cargo-sync-rdme badge [[ -->
![License: MIT OR Apache-2.0](https://img.shields.io/crates/l/scuffle-flv.svg?style=flat-square)
[![docs.rs](https://img.shields.io/docsrs/scuffle-flv.svg?logo=docs.rs&style=flat-square)](https://docs.rs/scuffle-flv)
[![crates.io](https://img.shields.io/crates/v/scuffle-flv.svg?logo=rust&style=flat-square)](https://crates.io/crates/scuffle-flv)
[![GitHub Actions: ci](https://img.shields.io/github/actions/workflow/status/scufflecloud/scuffle/ci.yaml.svg?label=ci&logo=github&style=flat-square)](https://github.com/scufflecloud/scuffle/actions/workflows/ci.yaml)
[![Codecov](https://img.shields.io/codecov/c/github/scufflecloud/scuffle.svg?label=codecov&logo=codecov&style=flat-square)](https://codecov.io/gh/scufflecloud/scuffle)
<!-- cargo-sync-rdme ]] -->

---

<!-- cargo-sync-rdme rustdoc [[ -->
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
<!-- cargo-sync-rdme ]] -->
