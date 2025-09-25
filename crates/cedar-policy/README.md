<!-- dprint-ignore-file -->
<!-- sync-readme title [[ -->
# scuffle-cedar-policy
<!-- sync-readme ]] -->

> [!WARNING]  
> This crate is under active development and may not be stable.

<!-- sync-readme badge [[ -->
[![docs.rs](https://img.shields.io/docsrs/scuffle-cedar-policy/0.1.0.svg?logo=docs.rs&label=docs.rs&style=flat-square)](https://docs.rs/scuffle-cedar-policy/0.1.0)
[![crates.io](https://img.shields.io/badge/crates.io-v0.1.0-orange?style=flat-square&logo=rust&logoColor=white)](https://crates.io/crates/scuffle-cedar-policy/0.1.0)
![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-purple.svg?style=flat-square)
![Crates.io Size](https://img.shields.io/crates/size/scuffle-cedar-policy/0.1.0.svg?style=flat-square)
![Crates.io Downloads](https://img.shields.io/crates/dv/scuffle-cedar-policy/0.1.0.svg?&label=downloads&style=flat-square)
[![Codecov](https://img.shields.io/codecov/c/github/scufflecloud/scuffle.svg?label=codecov&logo=codecov&style=flat-square)](https://app.codecov.io/gh/scufflecloud/scuffle)
<!-- sync-readme ]] -->

---

<!-- sync-readme rustdoc [[ -->
Cedar is a policy language used to express permisisons using a relationship model.

This crate extends the [`cedar-policy`](https://docs.rs/cedar-policy) crate by adding some type safe traits and
a code generator crate [`scuffle-cedar-policy-codegen`](https://docs.rs/scuffle-cedar-policy-codegen) which can be used
to generate types from a cedar schema file.

You can then use this in combo with cedar to have type-safe schema evaluation.

See the [changelog](./CHANGELOG.md) for a full release history.

### Feature flags

*No documented features in Cargo.toml*

### License

This project is licensed under the MIT or Apache-2.0 license.
You can choose between one of them if you use this work.

`SPDX-License-Identifier: MIT OR Apache-2.0`
<!-- sync-readme ]] -->
