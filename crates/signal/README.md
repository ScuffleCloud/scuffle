<!-- cargo-sync-rdme title [[ -->
# scuffle-signal
<!-- cargo-sync-rdme ]] -->

> [!WARNING]  
> This crate is under active development and may not be stable.

<!-- cargo-sync-rdme badge [[ -->
![License: MIT OR Apache-2.0](https://img.shields.io/crates/l/scuffle-signal.svg?style=flat-square)
[![docs.rs](https://img.shields.io/docsrs/scuffle-signal.svg?logo=docs.rs&style=flat-square)](https://docs.rs/scuffle-signal)
[![crates.io](https://img.shields.io/crates/v/scuffle-signal.svg?logo=rust&style=flat-square)](https://crates.io/crates/scuffle-signal)
[![GitHub Actions: ci](https://img.shields.io/github/actions/workflow/status/scufflecloud/scuffle/ci.yaml.svg?label=ci&logo=github&style=flat-square)](https://github.com/scufflecloud/scuffle/actions/workflows/ci.yaml)
[![Codecov](https://img.shields.io/codecov/c/github/scufflecloud/scuffle.svg?label=codecov&logo=codecov&style=flat-square)](https://codecov.io/gh/scufflecloud/scuffle)
<!-- cargo-sync-rdme ]] -->

---

<!-- cargo-sync-rdme rustdoc [[ -->
A crate designed to provide a more user friendly interface to
`tokio::signal`.

See the [changelog](./CHANGELOG.md) for a full release history.

### Feature flags

* **`bootstrap`** —  Enables scuffle-bootstrap support
* **`docs`** —  Enables changelog and documentation of feature flags

### Why do we need this?

The `tokio::signal` module provides a way for us to wait for a signal to be
received in a non-blocking way. This crate extends that with a more helpful
interface allowing the ability to listen to multiple signals concurrently.

### Example

````rust
use scuffle_signal::SignalHandler;
use tokio::signal::unix::SignalKind;

let mut handler = SignalHandler::new()
    .with_signal(SignalKind::interrupt())
    .with_signal(SignalKind::terminate());

// Wait for a signal to be received
let signal = handler.await;

// Handle the signal
let interrupt = SignalKind::interrupt();
let terminate = SignalKind::terminate();
match signal {
    interrupt => {
        // Handle SIGINT
        println!("received SIGINT");
    },
    terminate => {
        // Handle SIGTERM
        println!("received SIGTERM");
    },
}
````

### License

This project is licensed under the MIT or Apache-2.0 license.
You can choose between one of them if you use this work.

`SPDX-License-Identifier: MIT OR Apache-2.0`
<!-- cargo-sync-rdme ]] -->
