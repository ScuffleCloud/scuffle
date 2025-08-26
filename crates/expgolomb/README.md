<!-- dprint-ignore-file -->
<!-- sync-readme title [[ -->
# scuffle-expgolomb
<!-- sync-readme ]] -->

> [!WARNING]  
> This crate is under active development and may not be stable.

<!-- sync-readme badge [[ -->
[![docs.rs](https://img.shields.io/docsrs/scuffle-expgolomb/0.1.5.svg?logo=docs.rs&label=docs.rs&style=flat-square)](https://docs.rs/scuffle-expgolomb/0.1.5)
[![crates.io](https://img.shields.io/badge/crates.io-v0.1.5-orange?style=flat-square&logo=rust&logoColor=white)](https://crates.io/crates/scuffle-expgolomb/0.1.5)
![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-purple.svg?style=flat-square)
![Crates.io Size](https://img.shields.io/crates/size/scuffle-expgolomb/0.1.5.svg?style=flat-square)
![Crates.io Downloads](https://img.shields.io/crates/dv/scuffle-expgolomb/0.1.5.svg?&label=downloads&style=flat-square)
[![Codecov](https://img.shields.io/codecov/c/github/scufflecloud/scuffle.svg?label=codecov&logo=codecov&style=flat-square)](https://app.codecov.io/gh/scufflecloud/scuffle)
<!-- sync-readme ]] -->

---

<!-- sync-readme rustdoc [[ -->
A set of helper functions to encode and decode exponential-golomb values.

This crate extends upon the [`BitReader`](https://docs.rs/scuffle_bytes_util/0.1.5/scuffle_bytes_util/bit_read/struct.BitReader.html) and [`BitWriter`](https://docs.rs/scuffle_bytes_util/0.1.5/scuffle_bytes_util/bit_write/struct.BitWriter.html) from the
[`scuffle-bytes-util`](https://docs.rs/scuffle_bytes_util/0.1.5/scuffle_bytes_util/index.html) crate to provide functionality
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
<!-- sync-readme ]] -->
