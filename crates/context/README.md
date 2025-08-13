<!-- cargo-sync-rdme title [[ -->
# scuffle-context
<!-- cargo-sync-rdme ]] -->

> [!WARNING]  
> This crate is under active development and may not be stable.

<!-- cargo-sync-rdme badge [[ -->
![License: MIT OR Apache-2.0](https://img.shields.io/crates/l/scuffle-context.svg?style=flat-square)
[![docs.rs](https://img.shields.io/docsrs/scuffle-context.svg?logo=docs.rs&style=flat-square)](https://docs.rs/scuffle-context)
[![crates.io](https://img.shields.io/crates/v/scuffle-context.svg?logo=rust&style=flat-square)](https://crates.io/crates/scuffle-context)
[![GitHub Actions: ci](https://img.shields.io/github/actions/workflow/status/scufflecloud/scuffle/ci.yaml.svg?label=ci&logo=github&style=flat-square)](https://github.com/scufflecloud/scuffle/actions/workflows/ci.yaml)
[![Codecov](https://img.shields.io/codecov/c/github/scufflecloud/scuffle.svg?label=codecov&logo=codecov&style=flat-square)](https://codecov.io/gh/scufflecloud/scuffle)
<!-- cargo-sync-rdme ]] -->

---

<!-- cargo-sync-rdme rustdoc [[ -->
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
<!-- cargo-sync-rdme ]] -->
