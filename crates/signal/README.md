<!-- sync-readme title [[ -->
# scuffle-signal
<!-- sync-readme ]] -->

> [!WARNING]  
> This crate is under active development and may not be stable.

<!-- sync-readme badge [[ -->
[![docs.rs](https://img.shields.io/docsrs/scuffle-signal/0.3.3.svg?logo=docs.rs&label=docs.rs&style=flat-square)](https://docs.rs/scuffle-signal/0.3.3)
[![crates.io](https://img.shields.io/badge/crates.io-v0.3.3-orange?style=flat-square&logo=rust&logoColor=white)](https://crates.io/crates/scuffle-signal/0.3.3)
![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-purple.svg?style=flat-square)
![Crates.io Size](https://img.shields.io/crates/size/scuffle-signal/0.3.3.svg?style=flat-square)
![Crates.io Downloads](https://img.shields.io/crates/dv/scuffle-signal/0.3.3.svg?&label=downloads&style=flat-square)
[![Codecov](https://img.shields.io/codecov/c/github/scufflecloud/scuffle.svg?label=codecov&logo=codecov&style=flat-square)](https://app.codecov.io/gh/scufflecloud/scuffle)
<!-- sync-readme ]] -->

---

<!-- sync-readme rustdoc [[ -->
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
<!-- sync-readme ]] -->
