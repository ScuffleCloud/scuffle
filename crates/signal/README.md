# scuffle-signal

> [!WARNING]  
> This crate is under active development and may not be stable.

[![crates.io](https://img.shields.io/crates/v/scuffle-signal.svg)](https://crates.io/crates/scuffle-signal) [![docs.rs](https://img.shields.io/docsrs/scuffle-signal)](https://docs.rs/scuffle-signal)

---

A crate designed to provide a more user friendly interface to `tokio::signal`.

## Why do we need this?

The `tokio::signal` module provides a way for us to wait for a signal to be received in a non-blocking way.
This crate extends that with a more helpful interface allowing the ability to listen to multiple signals concurrently.

## Example

```rust
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
    }
    terminate => {
        // Handle SIGTERM
        println!("received SIGTERM");
    }
}
```

## Status

This crate is currently under development and is not yet stable.

Unit tests are not yet fully implemented. Use at your own risk.

## License

This project is licensed under the [MIT](./LICENSE.MIT) or [Apache-2.0](./LICENSE.Apache-2.0) license.
You can choose between one of them if you use this work.

`SPDX-License-Identifier: MIT OR Apache-2.0`
