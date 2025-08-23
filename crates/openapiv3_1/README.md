<!-- dprint-ignore-file -->
<!-- sync-readme title [[ -->
# openapiv3_1
<!-- sync-readme ]] -->

> [!WARNING]  
> This crate is under active development and may not be stable.

<!-- sync-readme badge [[ -->
[![docs.rs](https://img.shields.io/docsrs/openapiv3_1/0.1.3.svg?logo=docs.rs&label=docs.rs&style=flat-square)](https://docs.rs/openapiv3_1/0.1.3)
[![crates.io](https://img.shields.io/badge/crates.io-v0.1.3-orange?style=flat-square&logo=rust&logoColor=white)](https://crates.io/crates/openapiv3_1/0.1.3)
![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-purple.svg?style=flat-square)
![Crates.io Size](https://img.shields.io/crates/size/openapiv3_1/0.1.3.svg?style=flat-square)
![Crates.io Downloads](https://img.shields.io/crates/dv/openapiv3_1/0.1.3.svg?&label=downloads&style=flat-square)
[![Codecov](https://img.shields.io/codecov/c/github/scufflecloud/scuffle.svg?label=codecov&logo=codecov&style=flat-square)](https://app.codecov.io/gh/scufflecloud/scuffle)
<!-- sync-readme ]] -->

---

<!-- sync-readme rustdoc [[ -->
Rust implementation of OpenAPI Spec v3.1.x

A lof the code was taken from [`utoipa`](https://crates.io/crates/utoipa).

The main difference is the full JSON Schema 2020-12 Definitions.

See the [changelog](./CHANGELOG.md) for a full release history.

### Feature flags

* **`docs`** —  Enables changelog and documentation of feature flags
* **`debug`** —  Enable derive(Debug) on all types
* **`yaml`** —  Enables `to_yaml` function.

### Alternatives

* [`openapiv3`](https://crates.io/crates/openapiv3): Implements the openapi v3.0.x spec, does not implement full json schema draft 2020-12 spec.
* [`utoipa`](https://crates.io/crates/utoipa): A fully fletched openapi-type-generator implementing some of the v3.1.x spec.
* [`schemars`](https://crates.io/crates/schemars): A fully fletched jsonschema-type-generator implementing some of the json schema draft 2020-12 spec.

### License

This project is licensed under the MIT or Apache-2.0 license.
You can choose between one of them if you use this work.

`SPDX-License-Identifier: MIT OR Apache-2.0`
<!-- sync-readme ]] -->
