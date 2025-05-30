<!-- cargo-sync-rdme title [[ -->
# tinc-build
<!-- cargo-sync-rdme ]] -->

> [!WARNING]  
> This crate is under active development and may not be stable.

<!-- cargo-sync-rdme badge [[ -->
![License: MIT OR Apache-2.0](https://img.shields.io/crates/l/tinc-build.svg?style=flat-square)
[![docs.rs](https://img.shields.io/docsrs/tinc-build.svg?logo=docs.rs&style=flat-square)](https://docs.rs/tinc-build)
[![crates.io](https://img.shields.io/crates/v/tinc-build.svg?logo=rust&style=flat-square)](https://crates.io/crates/tinc-build)
[![GitHub Actions: ci](https://img.shields.io/github/actions/workflow/status/scufflecloud/scuffle/ci.yaml.svg?label=ci&logo=github&style=flat-square)](https://github.com/scufflecloud/scuffle/actions/workflows/ci.yaml)
[![Codecov](https://img.shields.io/codecov/c/github/scufflecloud/scuffle.svg?label=codecov&logo=codecov&style=flat-square)](https://codecov.io/gh/scufflecloud/scuffle)
<!-- cargo-sync-rdme ]] -->

---

<!-- cargo-sync-rdme rustdoc [[ -->
The code generator for [`tinc`](https://crates.io/crates/tinc).

### Feature flags

* **`prost`** *(enabled by default)* —  Enables prost codegen
* **`docs`** —  Enables documentation of feature flags

### Usage

In your `build.rs`:

````rust,no_run
fn main() {
    tinc_build::Config::prost()
        .compile_protos(&["proto/test.proto"], &["proto"])
        .unwrap();
}
````

Look at [`Config`](https://docs.rs/tinc-build/0.1.3/tinc_build/struct.Config.html) to see different options to configure the generator.

### License

This project is licensed under the MIT or Apache-2.0 license.
You can choose between one of them if you use this work.

`SPDX-License-Identifier: MIT OR Apache-2.0`
<!-- cargo-sync-rdme ]] -->
