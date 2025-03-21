# scuffle-flv

> [!WARNING]  
> This crate is under active development and may not be stable.

[![crates.io](https://img.shields.io/crates/v/scuffle-flv.svg)](https://crates.io/crates/scuffle-flv) [![docs.rs](https://img.shields.io/docsrs/scuffle-flv)](https://docs.rs/scuffle-flv)

---

A pure Rust implementation of the FLV format, allowing for demuxing of FLV
files and streams.

## Specifications

| Name | Version | Link | Comments |
| --- | --- | --- | --- |
| Video File Format Specification | `10` | <https://github.com/veovera/enhanced-rtmp/blob/main/docs/legacy/video-file-format-v10-0-spec.pdf> | |
| Adobe Flash Video File Format Specification | `10.1` | <https://github.com/veovera/enhanced-rtmp/blob/main/docs/legacy/video-file-format-v10-1-spec.pdf> | Refered to as 'Legacy FLV spec' in this documentation |
| Enhancing RTMP, FLV | `v1-2024-02-29-r1` | <https://github.com/veovera/enhanced-rtmp/blob/main/docs/enhanced/enhanced-rtmp-v1.pdf> | |
| Enhanced RTMP | `v2-2024-10-22-b1` | <https://github.com/veovera/enhanced-rtmp/blob/main/docs/enhanced/enhanced-rtmp-v2.pdf> | Refered to as 'Enhanced RTMP spec' in this documentation |

## License

This project is licensed under the [MIT](./LICENSE.MIT) or [Apache-2.0](./LICENSE.Apache-2.0) license.
You can choose between one of them if you use this work.

`SPDX-License-Identifier: MIT OR Apache-2.0`
