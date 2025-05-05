//! A wrapper around opentelemetry to provide a more ergonomic interface for
//! creating metrics.
//!
//! This crate can be used together with the
//! [`scuffle-bootstrap-telemetry`](../scuffle_bootstrap_telemetry) crate
//! which provides a service that integrates with the
//! [`scuffle-bootstrap`](../scuffle_bootstrap) ecosystem.
//!
//! ## Example
//!
//! ```rust
//! #[scuffle_metrics::metrics]
//! mod example {
//!     use scuffle_metrics::{MetricEnum, collector::CounterU64};
//!
//!     #[derive(MetricEnum)]
//!     pub enum Kind {
//!         Http,
//!         Grpc,
//!     }
//!
//!     #[metrics(unit = "requests")]
//!     pub fn request(kind: Kind) -> CounterU64;
//! }
//!
//! // Increment the counter
//! example::request(example::Kind::Http).incr();
//! ```
//!
//! For details see [`metrics!`](metrics).
//!
//! ## Status
//!
//! This crate is currently under development and is not yet stable.
//!
//! Unit tests are not yet fully implemented. Use at your own risk.
//!
//! ## License
//!
//! This project is licensed under the [MIT](./LICENSE.MIT) or
//! [Apache-2.0](./LICENSE.Apache-2.0) license. You can choose between one of
//! them if you use this work.
//!
//! `SPDX-License-Identifier: MIT OR Apache-2.0`
#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(missing_docs)]
#![deny(unreachable_pub)]
#![deny(clippy::undocumented_unsafe_blocks)]
#![deny(clippy::multiple_unsafe_ops_per_block)]

/// A copy of the opentelemetry-prometheus crate, updated to work with the
/// latest version of opentelemetry.
#[cfg(feature = "prometheus")]
#[cfg_attr(docsrs, doc(cfg(feature = "prometheus")))]
pub mod prometheus;

#[doc(hidden)]
pub mod value;

pub mod collector;

pub use collector::{
    CounterF64, CounterU64, GaugeF64, GaugeI64, GaugeU64, HistogramF64, HistogramU64, UpDownCounterF64, UpDownCounterI64,
};
pub use opentelemetry;
pub use scuffle_metrics_derive::{MetricEnum, metrics};

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use std::sync::Arc;

    use opentelemetry::{Key, KeyValue, Value};
    use opentelemetry_sdk::Resource;
    use opentelemetry_sdk::metrics::data::{ResourceMetrics, Sum};
    use opentelemetry_sdk::metrics::reader::MetricReader;
    use opentelemetry_sdk::metrics::{ManualReader, ManualReaderBuilder, SdkMeterProvider};

    #[test]
    fn derive_enum() {
        insta::assert_snapshot!(postcompile::compile! {
            use scuffle_metrics::MetricEnum;

            #[derive(MetricEnum)]
            pub enum Kind {
                Http,
                Grpc,
            }
        });
    }

    #[test]
    fn opentelemetry() {
        #[derive(Debug, Clone)]
        struct TestReader(Arc<ManualReader>);

        impl TestReader {
            fn new() -> Self {
                Self(Arc::new(ManualReaderBuilder::new().build()))
            }

            fn read(&self) -> ResourceMetrics {
                let mut metrics = ResourceMetrics {
                    resource: Resource::builder_empty().build(),
                    scope_metrics: vec![],
                };

                self.0.collect(&mut metrics).expect("collect");

                metrics
            }
        }

        impl opentelemetry_sdk::metrics::reader::MetricReader for TestReader {
            fn register_pipeline(&self, pipeline: std::sync::Weak<opentelemetry_sdk::metrics::Pipeline>) {
                self.0.register_pipeline(pipeline)
            }

            fn collect(
                &self,
                rm: &mut opentelemetry_sdk::metrics::data::ResourceMetrics,
            ) -> opentelemetry_sdk::metrics::MetricResult<()> {
                self.0.collect(rm)
            }

            fn force_flush(&self) -> opentelemetry_sdk::error::OTelSdkResult {
                self.0.force_flush()
            }

            fn shutdown(&self) -> opentelemetry_sdk::error::OTelSdkResult {
                self.0.shutdown()
            }

            fn temporality(
                &self,
                kind: opentelemetry_sdk::metrics::InstrumentKind,
            ) -> opentelemetry_sdk::metrics::Temporality {
                self.0.temporality(kind)
            }
        }

        #[crate::metrics(crate_path = "crate")]
        mod example {
            use crate::{CounterU64, MetricEnum};

            #[derive(MetricEnum)]
            #[metrics(crate_path = "crate")]
            pub enum Kind {
                Http,
                Grpc,
            }

            #[metrics(unit = "requests")]
            pub fn request(kind: Kind) -> CounterU64;
        }

        let reader = TestReader::new();
        let provider = SdkMeterProvider::builder()
            .with_resource(
                Resource::builder()
                    .with_attribute(KeyValue::new("service.name", "test_service"))
                    .build(),
            )
            .with_reader(reader.clone())
            .build();
        opentelemetry::global::set_meter_provider(provider);

        let metrics = reader.read();

        assert!(!metrics.resource.is_empty());
        assert_eq!(
            metrics.resource.get(&Key::from_static_str("service.name")),
            Some(Value::from("test_service"))
        );
        assert_eq!(
            metrics.resource.get(&Key::from_static_str("telemetry.sdk.name")),
            Some(Value::from("opentelemetry"))
        );
        assert!(metrics.resource.get(&Key::from_static_str("telemetry.sdk.version")).is_some());
        assert_eq!(
            metrics.resource.get(&Key::from_static_str("telemetry.sdk.language")),
            Some(Value::from("rust"))
        );

        assert!(metrics.scope_metrics.is_empty());

        example::request(example::Kind::Http).incr();

        let metrics = reader.read();

        assert_eq!(metrics.scope_metrics.len(), 1);
        assert_eq!(metrics.scope_metrics[0].scope.name(), "scuffle-metrics");
        assert!(metrics.scope_metrics[0].scope.version().is_some());
        assert_eq!(metrics.scope_metrics[0].metrics.len(), 1);
        assert_eq!(metrics.scope_metrics[0].metrics[0].name, "example_request");
        assert_eq!(metrics.scope_metrics[0].metrics[0].description, "");
        assert_eq!(metrics.scope_metrics[0].metrics[0].unit, "requests");
        let sum: &Sum<u64> = metrics.scope_metrics[0].metrics[0]
            .data
            .as_any()
            .downcast_ref()
            .expect("wrong data type");
        assert_eq!(sum.temporality, opentelemetry_sdk::metrics::Temporality::Cumulative);
        assert!(sum.is_monotonic);
        assert_eq!(sum.data_points.len(), 1);
        assert_eq!(sum.data_points[0].value, 1);
        assert_eq!(sum.data_points[0].attributes.len(), 1);
        assert_eq!(sum.data_points[0].attributes[0].key, Key::from_static_str("kind"));
        assert_eq!(sum.data_points[0].attributes[0].value, Value::from("Http"));

        example::request(example::Kind::Http).incr();

        let metrics = reader.read();

        assert_eq!(metrics.scope_metrics.len(), 1);
        assert_eq!(metrics.scope_metrics[0].metrics.len(), 1);
        let sum: &Sum<u64> = metrics.scope_metrics[0].metrics[0]
            .data
            .as_any()
            .downcast_ref()
            .expect("wrong data type");
        assert_eq!(sum.data_points.len(), 1);
        assert_eq!(sum.data_points[0].value, 2);
        assert_eq!(sum.data_points[0].attributes.len(), 1);
        assert_eq!(sum.data_points[0].attributes[0].key, Key::from_static_str("kind"));
        assert_eq!(sum.data_points[0].attributes[0].value, Value::from("Http"));

        example::request(example::Kind::Grpc).incr();

        let metrics = reader.read();

        assert_eq!(metrics.scope_metrics.len(), 1);
        assert_eq!(metrics.scope_metrics[0].metrics.len(), 1);
        let sum: &Sum<u64> = metrics.scope_metrics[0].metrics[0]
            .data
            .as_any()
            .downcast_ref()
            .expect("wrong data type");
        assert_eq!(sum.data_points.len(), 2);
        let grpc = sum
            .data_points
            .iter()
            .find(|dp| {
                dp.attributes.len() == 1
                    && dp.attributes[0].key == Key::from_static_str("kind")
                    && dp.attributes[0].value == Value::from("Grpc")
            })
            .expect("grpc data point not found");
        assert_eq!(grpc.value, 1);
    }
}
