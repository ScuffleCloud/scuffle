<!-- cargo-sync-rdme title [[ -->
# isobmff
<!-- cargo-sync-rdme ]] -->

> [!WARNING]  
> This crate is under active development and may not be stable.

<!-- cargo-sync-rdme badge [[ -->
![License: MIT OR Apache-2.0](https://img.shields.io/crates/l/isobmff.svg?style=flat-square)
[![docs.rs](https://img.shields.io/docsrs/isobmff.svg?logo=docs.rs&style=flat-square)](https://docs.rs/isobmff)
[![crates.io](https://img.shields.io/crates/v/isobmff.svg?logo=rust&style=flat-square)](https://crates.io/crates/isobmff)
[![GitHub Actions: ci](https://img.shields.io/github/actions/workflow/status/scufflecloud/scuffle/ci.yaml.svg?label=ci&logo=github&style=flat-square)](https://github.com/scufflecloud/scuffle/actions/workflows/ci.yaml)
[![Codecov](https://img.shields.io/codecov/c/github/scufflecloud/scuffle.svg?label=codecov&logo=codecov&style=flat-square)](https://codecov.io/gh/scufflecloud/scuffle)
<!-- cargo-sync-rdme ]] -->

---

<!-- cargo-sync-rdme rustdoc [[ -->
Implementation of the ISO Base Media File Format (ISOBMFF) defined by ISO/IEC 14496-12.

### Example

TODO

### Notes

This implementation does not preserve the order of boxes when remuxing files and individual boxes.
Instead it uses the recommended box order as defined in ISO/IEC 14496-12 - 6.3.4.

See the [changelog](./CHANGELOG.md) for a full release history.

### Feature flags

* **`docs`** —  Enables changelog and documentation of feature flags

### License

This project is licensed under the [MIT](./LICENSE.MIT) or [Apache-2.0](./LICENSE.Apache-2.0) license.
You can choose between one of them if you use this work.

`SPDX-License-Identifier: MIT OR Apache-2.0`
<!-- cargo-sync-rdme ]] -->
