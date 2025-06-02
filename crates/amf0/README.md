<!-- cargo-sync-rdme title [[ -->
# scuffle-amf0
<!-- cargo-sync-rdme ]] -->

> [!WARNING]  
> This crate is under active development and may not be stable.

<!-- cargo-sync-rdme badge [[ -->
![License: MIT OR Apache-2.0](https://img.shields.io/crates/l/scuffle-amf0.svg?style=flat-square)
[![docs.rs](https://img.shields.io/docsrs/scuffle-amf0.svg?logo=docs.rs&style=flat-square)](https://docs.rs/scuffle-amf0)
[![crates.io](https://img.shields.io/crates/v/scuffle-amf0.svg?logo=rust&style=flat-square)](https://crates.io/crates/scuffle-amf0)
[![GitHub Actions: ci](https://img.shields.io/github/actions/workflow/status/scufflecloud/scuffle/ci.yaml.svg?label=ci&logo=github&style=flat-square)](https://github.com/scufflecloud/scuffle/actions/workflows/ci.yaml)
[![Codecov](https://img.shields.io/codecov/c/github/scufflecloud/scuffle.svg?label=codecov&logo=codecov&style=flat-square)](https://codecov.io/gh/scufflecloud/scuffle)
<!-- cargo-sync-rdme ]] -->

---

<!-- cargo-sync-rdme rustdoc [[ -->
A pure-rust implementation of AMF0 encoder and decoder.

This crate provides serde support for serialization and deserialization of AMF0 data.

See the [changelog](./CHANGELOG.md) for a full release history.

### Feature flags

* **`serde`** —  Enables serde support
* **`docs`** —  Enables changelog and documentation of feature flags
* **`preserve_order`** —  Use an index map instead of a btree-map for Amf0Objects

### Specification

|Name|Version|Link|Comments|
|----|-------|----|--------|
|Action Message Format – AMF 0|-|<https://rtmp.veriskope.com/pdf/amf0-file-format-specification.pdf>|Refered to as ‘AMF0 spec’ in this documentation|

### Limitations

* Does not support AMF0 references.
* Does not support the AVM+ Type Marker. (see AMF 0 spec, 3.1)

### Example

````rust
// Decode a string value from bytes
let value: String = scuffle_amf0::from_slice(bytes)?;

// .. do something with the value

// Encode a value into a writer
scuffle_amf0::to_writer(&mut writer, &value)?;
````

### License

This project is licensed under the MIT or Apache-2.0 license.
You can choose between one of them if you use this work.

`SPDX-License-Identifier: MIT OR Apache-2.0`
<!-- cargo-sync-rdme ]] -->
