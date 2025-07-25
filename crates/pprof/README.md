<!-- sync-readme title [[ -->
# scuffle-pprof
<!-- sync-readme ]] -->

> [!WARNING]  
> This crate is under active development and may not be stable.

<!-- sync-readme badge [[ -->
[![docs.rs](https://img.shields.io/docsrs/scuffle-pprof/0.2.0.svg?logo=docs.rs&label=docs.rs&style=flat-square)](https://docs.rs/scuffle-pprof/0.2.0)
[![crates.io](https://img.shields.io/badge/crates.io-v0.2.0-orange?style=flat-square&logo=rust&logoColor=white)](https://crates.io/crates/scuffle-pprof/0.2.0)
![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-purple.svg?style=flat-square)
![Crates.io Size](https://img.shields.io/crates/size/scuffle-pprof/0.2.0.svg?style=flat-square)
![Crates.io Downloads](https://img.shields.io/crates/dv/scuffle-pprof/0.2.0.svg?&label=downloads&style=flat-square)
[![Codecov](https://img.shields.io/codecov/c/github/scufflecloud/scuffle.svg?label=codecov&logo=codecov&style=flat-square)](https://app.codecov.io/gh/scufflecloud/scuffle)
<!-- sync-readme ]] -->

---

<!-- sync-readme rustdoc [[ -->
A crate designed to provide a more ergonomic interface to the `pprof` crate.

Only supports Unix-like systems. This crate will be empty on Windows.

See the [changelog](./CHANGELOG.md) for a full release history.

### Feature flags

* **`docs`** —  Enables changelog and documentation of feature flags

### Example

````rust,no_run
// Create a new CPU profiler with a sampling frequency of 1000 Hz and an empty ignore list.
let cpu = scuffle_pprof::Cpu::new::<String>(1000, &[]);

// Capture a pprof profile for 10 seconds.
// This call is blocking. It is recommended to run it in a separate thread.
let capture = cpu.capture(std::time::Duration::from_secs(10)).unwrap();

// Write the profile to a file.
std::fs::write("capture.pprof", capture).unwrap();
````

### Analyzing the profile

The resulting profile can be analyzed using the [`pprof`](https://github.com/google/pprof) tool.

For example, to generate a flamegraph:

````sh
pprof -svg capture.pprof
````

### License

This project is licensed under the MIT or Apache-2.0 license.
You can choose between one of them if you use this work.

`SPDX-License-Identifier: MIT OR Apache-2.0`
<!-- sync-readme ]] -->
