<!-- cargo-sync-rdme title [[ -->
# scuffle-bootstrap-telemetry
<!-- cargo-sync-rdme ]] -->

> [!WARNING]  
> This crate is under active development and may not be stable.

<!-- cargo-sync-rdme badge [[ -->
![License: MIT OR Apache-2.0](https://img.shields.io/crates/l/scuffle-bootstrap-telemetry.svg?style=flat-square)
[![docs.rs](https://img.shields.io/docsrs/scuffle-bootstrap-telemetry.svg?logo=docs.rs&style=flat-square)](https://docs.rs/scuffle-bootstrap-telemetry)
[![crates.io](https://img.shields.io/crates/v/scuffle-bootstrap-telemetry.svg?logo=rust&style=flat-square)](https://crates.io/crates/scuffle-bootstrap-telemetry)
[![GitHub Actions: ci](https://img.shields.io/github/actions/workflow/status/scufflecloud/scuffle/ci.yaml.svg?label=ci&logo=github&style=flat-square)](https://github.com/scufflecloud/scuffle/actions/workflows/ci.yaml)
[![Codecov](https://img.shields.io/codecov/c/github/scufflecloud/scuffle.svg?label=codecov&logo=codecov&style=flat-square)](https://codecov.io/gh/scufflecloud/scuffle)
<!-- cargo-sync-rdme ]] -->

---

<!-- cargo-sync-rdme rustdoc [[ -->
A crate used to add telemetry to applications built with the
[`scuffle-bootstrap`][scuffle_bootstrap] crate.

Emit metrics using the [`scuffle-metrics`][scuffle_metrics]
crate.

See the [changelog](./CHANGELOG.md) for a full release history.

### Feature flags

* **`prometheus`** *(enabled by default)* —  Enables prometheus support
* **`pprof`** *(enabled by default)* —  Enables pprof profiling
* **`opentelemetry`** *(enabled by default)* —  Enables opentelemetry
* **`opentelemetry-metrics`** *(enabled by default)* —  Enables opentelemetry metricx exporting
* **`opentelemetry-traces`** *(enabled by default)* —  Enables opentelemetry trace exporting
* **`opentelemetry-logs`** *(enabled by default)* —  Enables opentelemetry log exporting
* **`docs`** —  Enables changelog and documentation of feature flags
  See [`TelemetrySvc`](https://docs.rs/scuffle-bootstrap-telemetry/0.2.4/scuffle_bootstrap_telemetry/struct.TelemetrySvc.html) for more details.

### Example

````rust
use std::net::SocketAddr;
use std::sync::Arc;

use scuffle_bootstrap::global::GlobalWithoutConfig;
use scuffle_bootstrap_telemetry::{
    prometheus_client,
    opentelemetry,
    opentelemetry_sdk,
    TelemetryConfig,
    TelemetrySvc
};

struct Global {
    prometheus: prometheus_client::registry::Registry,
    open_telemetry: opentelemetry::OpenTelemetry,
}

impl GlobalWithoutConfig for Global {
    async fn init() -> anyhow::Result<Arc<Self>> {
        // Initialize the Prometheus metrics registry.
        let mut prometheus = prometheus_client::registry::Registry::default();
        // The exporter converts opentelemetry metrics into the Prometheus format.
        let exporter = scuffle_metrics::prometheus::exporter().build();
        // Register the exporter as a data source for the Prometheus registry.
        prometheus.register_collector(exporter.collector());

        // Initialize the OpenTelemetry metrics provider and add the Prometheus exporter as a reader.
        let metrics = opentelemetry_sdk::metrics::SdkMeterProvider::builder()
            .with_reader(exporter)
            .build();
        opentelemetry::global::set_meter_provider(metrics.clone());

        // Initialize the OpenTelemetry configuration instance.
        let open_telemetry = opentelemetry::OpenTelemetry::new().with_metrics(metrics);

        Ok(Arc::new(Self {
            prometheus,
            open_telemetry,
        }))
    }
}

impl TelemetryConfig for Global {
    fn bind_address(&self) -> Option<SocketAddr> {
        // Tells the http server to bind to port 8080 on localhost.
        Some(SocketAddr::from(([127, 0, 0, 1], 8080)))
    }

    fn prometheus_metrics_registry(&self) -> Option<&prometheus_client::registry::Registry> {
        Some(&self.prometheus)
    }

    fn opentelemetry(&self) -> Option<&opentelemetry::OpenTelemetry> {
        Some(&self.open_telemetry)
    }
}

#[scuffle_metrics::metrics]
mod example {
    use scuffle_metrics::{CounterU64, MetricEnum};

    #[derive(MetricEnum)]
    pub enum Kind {
        Http,
        Grpc,
    }

    #[metrics(unit = "requests")]
    pub fn request(kind: Kind) -> CounterU64;
}

// Now emit metrics from anywhere in your code using the `example` module.
example::request(example::Kind::Http).incr();

scuffle_bootstrap::main! {
    Global {
        TelemetrySvc,
    }
};
````

### License

This project is licensed under the MIT or Apache-2.0 license.
You can choose between one of them if you use this work.

`SPDX-License-Identifier: MIT OR Apache-2.0`

[scuffle_bootstrap]: https://docs.rs/scuffle-bootstrap
[scuffle_metrics]: https://docs.rs/scuffle-metrics
<!-- cargo-sync-rdme ]] -->
