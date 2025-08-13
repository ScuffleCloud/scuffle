<!-- cargo-sync-rdme title [[ -->
# scuffle-metrics
<!-- cargo-sync-rdme ]] -->

> [!WARNING]  
> This crate is under active development and may not be stable.

<!-- cargo-sync-rdme badge [[ -->
![License: MIT OR Apache-2.0](https://img.shields.io/crates/l/scuffle-metrics.svg?style=flat-square)
[![docs.rs](https://img.shields.io/docsrs/scuffle-metrics.svg?logo=docs.rs&style=flat-square)](https://docs.rs/scuffle-metrics)
[![crates.io](https://img.shields.io/crates/v/scuffle-metrics.svg?logo=rust&style=flat-square)](https://crates.io/crates/scuffle-metrics)
[![GitHub Actions: ci](https://img.shields.io/github/actions/workflow/status/scufflecloud/scuffle/ci.yaml.svg?label=ci&logo=github&style=flat-square)](https://github.com/scufflecloud/scuffle/actions/workflows/ci.yaml)
[![Codecov](https://img.shields.io/codecov/c/github/scufflecloud/scuffle.svg?label=codecov&logo=codecov&style=flat-square)](https://codecov.io/gh/scufflecloud/scuffle)
<!-- cargo-sync-rdme ]] -->

---

<!-- cargo-sync-rdme rustdoc [[ -->
A wrapper around opentelemetry to provide a more ergonomic interface for
creating metrics.

This crate can be used together with the [`scuffle-bootstrap-telemetry`](https://docs.rs/scuffle-bootstrap-telemetry) crate
which provides a service that integrates with the [`scuffle-bootstrap`](https://docs.rs/scuffle-bootstrap) ecosystem.

See the [changelog](./CHANGELOG.md) for a full release history.

### Feature flags

* **`prometheus`** *(enabled by default)* —  Enables prometheus support
* **`tracing`** —  Enables tracing support
* **`docs`** —  Enables changelog and documentation of feature flags

### Example

````rust
#[scuffle_metrics::metrics]
mod example {
    use scuffle_metrics::{MetricEnum, collector::CounterU64};

    #[derive(MetricEnum)]
    pub enum Kind {
        Http,
        Grpc,
    }

    #[metrics(unit = "requests")]
    pub fn request(kind: Kind) -> CounterU64;
}

// Increment the counter
example::request(example::Kind::Http).incr();
````

For details see [`metrics!`](.).

### License

This project is licensed under the MIT or Apache-2.0 license.
You can choose between one of them if you use this work.

`SPDX-License-Identifier: MIT OR Apache-2.0`
<!-- cargo-sync-rdme ]] -->
