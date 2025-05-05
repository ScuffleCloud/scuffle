# scuffle-amf0

> [!WARNING]  
> This crate is under active development and may not be stable.

[![crates.io](https://img.shields.io/crates/v/scuffle-amf0.svg)](https://crates.io/crates/scuffle-amf0) [![docs.rs](https://img.shields.io/docsrs/scuffle-amf0)](https://docs.rs/scuffle-amf0)

---

A pure-rust implementation of AMF0 encoder and decoder.

This crate provides serde support for serialization and deserialization of AMF0 data.

## Specification

| Name | Version | Link | Comments |
| --- | --- | --- | --- |
| Action Message Format -- AMF 0 | - | <https://rtmp.veriskope.com/pdf/amf0-file-format-specification.pdf> | Refered to as 'AMF0 spec' in this documentation |

## Limitations

- Does not support AMF0 references.
- Does not support the AVM+ Type Marker. (see AMF 0 spec, 3.1)

## Example

```rust
// Decode a string value from bytes
let value: String = scuffle_amf0::from_slice(bytes)?;

// .. do something with the value

// Encode a value into a writer
scuffle_amf0::to_writer(&mut writer, &value)?;
```

## Status

This crate is currently under development and is not yet stable.

Unit tests are not yet fully implemented. Use at your own risk.

## License

This project is licensed under the [MIT](./LICENSE.MIT) or [Apache-2.0](./LICENSE.Apache-2.0) license.
You can choose between one of them if you use this work.

`SPDX-License-Identifier: MIT OR Apache-2.0`
