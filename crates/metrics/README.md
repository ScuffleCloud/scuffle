<!-- sync-readme title [[ -->
# scuffle-metrics
<!-- sync-readme ]] -->

> [!WARNING]  
> This crate is under active development and may not be stable.

<!-- sync-readme badge [[ -->
[![docs.rs](https://img.shields.io/docsrs/scuffle-metrics/0.4.2.svg?logo=docs.rs&label=docs.rs&style=flat-square)](https://docs.rs/scuffle-metrics/0.4.2)
[![crates.io](https://img.shields.io/badge/crates.io-v0.4.2-orange?style=flat-square&logo=rust&logoColor=white)](https://crates.io/crates/scuffle-metrics/0.4.2)
![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-purple.svg?style=flat-square)
![Crates.io Size](https://img.shields.io/crates/size/scuffle-metrics/0.4.2.svg?style=flat-square)
![Crates.io Downloads](https://img.shields.io/crates/dv/scuffle-metrics/0.4.2.svg?&label=downloads&style=flat-square)
[![Codecov](https://img.shields.io/codecov/c/github/scufflecloud/scuffle.svg?label=codecov&logo=codecov&style=flat-square)](https://app.codecov.io/gh/scufflecloud/scuffle)
<!-- sync-readme ]] -->

---

<!-- sync-readme rustdoc [[ -->
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

For details see [`metrics!`](https://docs.rs/scuffle_metrics_derive/0.4.2/scuffle_metrics_derive/attr.metrics.html).

### License

This project is licensed under the MIT or Apache-2.0 license.
You can choose between one of them if you use this work.

`SPDX-License-Identifier: MIT OR Apache-2.0`
<!-- sync-readme ]] -->
