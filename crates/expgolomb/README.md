<!-- cargo-sync-rdme title [[ -->
# scuffle-expgolomb
<!-- cargo-sync-rdme ]] -->

> [!WARNING]  
> This crate is under active development and may not be stable.

<!-- cargo-sync-rdme badge [[ -->
![License: MIT OR Apache-2.0](https://img.shields.io/crates/l/scuffle-expgolomb.svg?style=flat-square)
[![docs.rs](https://img.shields.io/docsrs/scuffle-expgolomb.svg?logo=docs.rs&style=flat-square)](https://docs.rs/scuffle-expgolomb)
[![crates.io](https://img.shields.io/crates/v/scuffle-expgolomb.svg?logo=rust&style=flat-square)](https://crates.io/crates/scuffle-expgolomb)
[![GitHub Actions: ci](https://img.shields.io/github/actions/workflow/status/scufflecloud/scuffle/ci.yaml.svg?label=ci&logo=github&style=flat-square)](https://github.com/scufflecloud/scuffle/actions/workflows/ci.yaml)
[![Codecov](https://img.shields.io/codecov/c/github/scufflecloud/scuffle.svg?label=codecov&logo=codecov&style=flat-square)](https://codecov.io/gh/scufflecloud/scuffle)
<!-- cargo-sync-rdme ]] -->

---

<!-- cargo-sync-rdme rustdoc [[ -->
A set of helper functions to encode and decode exponential-golomb values.

This crate extends upon the \[`BitReader`\] and \[`BitWriter`\] from the
\[`scuffle-bytes-util`\]\[scuffle_bytes_util\] crate to provide functionality
for reading and writing Exp-Golomb encoded numbers.

See the [changelog](./CHANGELOG.md) for a full release history.

### Feature flags

* **`docs`** —  Enables changelog and documentation of feature flags

### Usage

````rust
use scuffle_expgolomb::{BitReaderExpGolombExt, BitWriterExpGolombExt};
use scuffle_bytes_util::{BitReader, BitWriter};

let mut bit_writer = BitWriter::default();
bit_writer.write_exp_golomb(0)?;
bit_writer.write_exp_golomb(1)?;
bit_writer.write_exp_golomb(2)?;

let data: Vec<u8> = bit_writer.finish()?;

let mut bit_reader = BitReader::new(std::io::Cursor::new(data));

let result = bit_reader.read_exp_golomb()?;
assert_eq!(result, 0);

let result = bit_reader.read_exp_golomb()?;
assert_eq!(result, 1);

let result = bit_reader.read_exp_golomb()?;
assert_eq!(result, 2);
````

### License

This project is licensed under the MIT or Apache-2.0 license.
You can choose between one of them if you use this work.

`SPDX-License-Identifier: MIT OR Apache-2.0`
<!-- cargo-sync-rdme ]] -->
