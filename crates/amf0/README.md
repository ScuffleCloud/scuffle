<!-- sync-readme title [[ -->
# scuffle-amf0
<!-- sync-readme ]] -->

> [!WARNING]  
> This crate is under active development and may not be stable.

<!-- sync-readme badge [[ -->
[![docs.rs](https://img.shields.io/docsrs/scuffle-amf0/0.2.4.svg?logo=docs.rs&label=docs.rs&style=flat-square)](https://docs.rs/scuffle-amf0/0.2.4)
[![crates.io](https://img.shields.io/badge/crates.io-v0.2.4-orange?style=flat-square&logo=rust&logoColor=white)](https://crates.io/crates/scuffle-amf0/0.2.4)
![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-purple.svg?style=flat-square)
![Crates.io Size](https://img.shields.io/crates/size/scuffle-amf0/0.2.4.svg?style=flat-square)
![Crates.io Downloads](https://img.shields.io/crates/dv/scuffle-amf0/0.2.4.svg?&label=downloads&style=flat-square)
[![Codecov](https://img.shields.io/codecov/c/github/scufflecloud/scuffle.svg?label=codecov&logo=codecov&style=flat-square)](https://app.codecov.io/gh/scufflecloud/scuffle)
<!-- sync-readme ]] -->

---

<!-- sync-readme rustdoc [[ -->
A pure-rust implementation of AMF0 encoder and decoder.

This crate provides serde support for serialization and deserialization of AMF0 data.

See the [changelog](./CHANGELOG.md) for a full release history.

### Feature flags

* **`serde`** —  Enables serde support
* **`docs`** —  Enables changelog and documentation of feature flags

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
<!-- sync-readme ]] -->
