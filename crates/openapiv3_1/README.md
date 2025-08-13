<!-- cargo-sync-rdme title [[ -->
# openapiv3_1
<!-- cargo-sync-rdme ]] -->

> [!WARNING]  
> This crate is under active development and may not be stable.

<!-- cargo-sync-rdme badge [[ -->
![License: MIT OR Apache-2.0](https://img.shields.io/crates/l/openapiv3_1.svg?style=flat-square)
[![docs.rs](https://img.shields.io/docsrs/openapiv3_1.svg?logo=docs.rs&style=flat-square)](https://docs.rs/openapiv3_1)
[![crates.io](https://img.shields.io/crates/v/openapiv3_1.svg?logo=rust&style=flat-square)](https://crates.io/crates/openapiv3_1)
[![GitHub Actions: ci](https://img.shields.io/github/actions/workflow/status/scufflecloud/scuffle/ci.yaml.svg?label=ci&logo=github&style=flat-square)](https://github.com/scufflecloud/scuffle/actions/workflows/ci.yaml)
[![Codecov](https://img.shields.io/codecov/c/github/scufflecloud/scuffle.svg?label=codecov&logo=codecov&style=flat-square)](https://codecov.io/gh/scufflecloud/scuffle)
<!-- cargo-sync-rdme ]] -->

---

<!-- cargo-sync-rdme rustdoc [[ -->
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
<!-- cargo-sync-rdme ]] -->
