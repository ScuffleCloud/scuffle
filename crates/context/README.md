<!-- dprint-ignore-file -->
<!-- sync-readme title [[ -->
# scuffle-context
<!-- sync-readme ]] -->

> [!WARNING]  
> This crate is under active development and may not be stable.

<!-- sync-readme badge [[ -->
[![docs.rs](https://img.shields.io/docsrs/scuffle-context/0.1.5.svg?logo=docs.rs&label=docs.rs&style=flat-square)](https://docs.rs/scuffle-context/0.1.5)
[![crates.io](https://img.shields.io/badge/crates.io-v0.1.5-orange?style=flat-square&logo=rust&logoColor=white)](https://crates.io/crates/scuffle-context/0.1.5)
![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-purple.svg?style=flat-square)
![Crates.io Size](https://img.shields.io/crates/size/scuffle-context/0.1.5.svg?style=flat-square)
![Crates.io Downloads](https://img.shields.io/crates/dv/scuffle-context/0.1.5.svg?&label=downloads&style=flat-square)
[![Codecov](https://img.shields.io/codecov/c/github/scufflecloud/scuffle.svg?label=codecov&logo=codecov&style=flat-square)](https://app.codecov.io/gh/scufflecloud/scuffle)
<!-- sync-readme ]] -->

---

<!-- sync-readme rustdoc [[ -->
A crate designed to provide the ability to cancel futures using a context
go-like approach, allowing for graceful shutdowns and cancellations.

See the [changelog](./CHANGELOG.md) for a full release history.

### Feature flags

* **`docs`** —  Enables changelog and documentation of feature flags

### Why do we need this?

Its often useful to wait for all the futures to shutdown or to cancel them
when we no longer care about the results. This crate provides an interface
to cancel all futures associated with a context or wait for them to finish
before shutting down. Allowing for graceful shutdowns and cancellations.

### Usage

Here is an example of how to use the `Context` to cancel a spawned task.

````rust
let (ctx, handler) = Context::new();

tokio::spawn(async {
    // Do some work
    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
}.with_context(ctx));

// Will stop the spawned task and cancel all associated futures.
handler.cancel();
````

### License

This project is licensed under the MIT or Apache-2.0 license.
You can choose between one of them if you use this work.

`SPDX-License-Identifier: MIT OR Apache-2.0`
<!-- sync-readme ]] -->
