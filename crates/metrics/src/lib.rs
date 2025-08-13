//! A wrapper around opentelemetry to provide a more ergonomic interface for
//! creating metrics.
//!
//! This crate can be used together with the [`scuffle-bootstrap-telemetry`](https://docs.rs/scuffle-bootstrap-telemetry) crate
//! which provides a service that integrates with the [`scuffle-bootstrap`](https://docs.rs/scuffle-bootstrap) ecosystem.
#![cfg_attr(feature = "docs", doc = "\n\nSee the [changelog][changelog] for a full release history.")]
#![cfg_attr(feature = "docs", doc = "## Feature flags")]
#![cfg_attr(feature = "docs", doc = document_features::document_features!())]
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
//! ## License
//!
//! This project is licensed under the MIT or Apache-2.0 license.
//! You can choose between one of them if you use this work.
//!
//! `SPDX-License-Identifier: MIT OR Apache-2.0`
#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![deny(missing_docs)]
#![deny(unreachable_pub)]
#![deny(clippy::undocumented_unsafe_blocks)]
#![deny(clippy::multiple_unsafe_ops_per_block)]

/// A copy of the opentelemetry-prometheus crate, updated to work with the
/// latest version of opentelemetry.
#[cfg(feature = "prometheus")]
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
    use opentelemetry_sdk::metrics::data::{AggregatedMetrics, MetricData, ResourceMetrics};
    use opentelemetry_sdk::metrics::reader::MetricReader;
    use opentelemetry_sdk::metrics::{ManualReader, ManualReaderBuilder, SdkMeterProvider};

    #[test]
    fn derive_enum() {
        insta::assert_snapshot!(postcompile::compile!({
            #[derive(scuffle_metrics::MetricEnum)]
            pub enum Kind {
                Http,
                Grpc,
            }
        }));
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
                let mut metrics = ResourceMetrics::default();

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
            ) -> opentelemetry_sdk::error::OTelSdkResult {
                self.0.collect(rm)
            }

            fn force_flush(&self) -> opentelemetry_sdk::error::OTelSdkResult {
                self.0.force_flush()
            }

            fn shutdown_with_timeout(&self, timeout: std::time::Duration) -> opentelemetry_sdk::error::OTelSdkResult {
                self.0.shutdown_with_timeout(timeout)
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

        assert!(!metrics.resource().is_empty());
        assert_eq!(
            metrics.resource().get(&Key::from_static_str("service.name")),
            Some(Value::from("test_service"))
        );
        assert_eq!(
            metrics.resource().get(&Key::from_static_str("telemetry.sdk.name")),
            Some(Value::from("opentelemetry"))
        );
        assert!(
            metrics
                .resource()
                .get(&Key::from_static_str("telemetry.sdk.version"))
                .is_some()
        );
        assert_eq!(
            metrics.resource().get(&Key::from_static_str("telemetry.sdk.language")),
            Some(Value::from("rust"))
        );

        assert!(metrics.scope_metrics().next().is_none());

        example::request(example::Kind::Http).incr();

        let metrics = reader.read();

        assert_eq!(metrics.scope_metrics().count(), 1);
        let scoped_metric = metrics.scope_metrics().next().unwrap();
        assert_eq!(scoped_metric.scope().name(), "scuffle-metrics");
        assert!(scoped_metric.scope().version().is_some());
        assert_eq!(scoped_metric.metrics().count(), 1);
        let scoped_metric_metric = scoped_metric.metrics().next().unwrap();
        assert_eq!(scoped_metric_metric.name(), "example_request");
        assert_eq!(scoped_metric_metric.description(), "");
        assert_eq!(scoped_metric_metric.unit(), "requests");
        let AggregatedMetrics::U64(MetricData::Sum(sum)) = scoped_metric_metric.data() else {
            unreachable!()
        };
        assert_eq!(sum.temporality(), opentelemetry_sdk::metrics::Temporality::Cumulative);
        assert!(sum.is_monotonic());
        assert_eq!(sum.data_points().count(), 1);
        let data_point = sum.data_points().next().unwrap();
        assert_eq!(data_point.value(), 1);
        assert_eq!(data_point.attributes().count(), 1);
        let attribute = data_point.attributes().next().unwrap();
        assert_eq!(attribute.key, Key::from_static_str("kind"));
        assert_eq!(attribute.value, Value::from("Http"));

        example::request(example::Kind::Http).incr();

        let metrics = reader.read();

        assert_eq!(metrics.scope_metrics().count(), 1);
        let scope_metric = metrics.scope_metrics().next().unwrap();
        assert_eq!(scope_metric.metrics().count(), 1);
        let scope_metric_metric = scope_metric.metrics().next().unwrap();
        let AggregatedMetrics::U64(MetricData::Sum(sum)) = scope_metric_metric.data() else {
            unreachable!()
        };
        assert_eq!(sum.data_points().count(), 1);
        let data_point = sum.data_points().next().unwrap();
        assert_eq!(data_point.value(), 2);
        assert_eq!(data_point.attributes().count(), 1);
        let attribute = data_point.attributes().next().unwrap();
        assert_eq!(attribute.key, Key::from_static_str("kind"));
        assert_eq!(attribute.value, Value::from("Http"));

        example::request(example::Kind::Grpc).incr();

        let metrics = reader.read();

        assert_eq!(metrics.scope_metrics().count(), 1);
        let scope_metric = metrics.scope_metrics().next().unwrap();
        assert_eq!(scope_metric.metrics().count(), 1);
        let scope_metric_metric = scope_metric.metrics().next().unwrap();
        let AggregatedMetrics::U64(MetricData::Sum(sum)) = scope_metric_metric.data() else {
            unreachable!()
        };
        assert_eq!(sum.data_points().count(), 2);
        let grpc = sum
            .data_points()
            .find(|dp| {
                dp.attributes().count() == 1
                    && dp.attributes().next().unwrap().key == Key::from_static_str("kind")
                    && dp.attributes().next().unwrap().value == Value::from("Grpc")
            })
            .expect("grpc data point not found");
        assert_eq!(grpc.value(), 1);
    }
}

/// Changelogs generated by [scuffle_changelog]
#[cfg(feature = "docs")]
#[scuffle_changelog::changelog]
pub mod changelog {}
