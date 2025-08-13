//! Metrics collectors.

use std::borrow::Cow;

use opentelemetry::KeyValue;

/// A helper trait to force the compiler to check that the collector is valid.
#[doc(hidden)]
pub trait IsCollector: private::Sealed {
    type Builder<'a>;

    fn builder(meter: &opentelemetry::metrics::Meter, name: impl Into<Cow<'static, str>>) -> Self::Builder<'_>;
}

mod private {
    pub trait Sealed {
        type Value;
    }
}

macro_rules! impl_collector {
    ($t:ty, $value:ty, $func:ident, $builder:ty) => {
        impl private::Sealed for $t {
            type Value = $value;
        }

        impl IsCollector for $t {
            type Builder<'a> = $builder;

            fn builder(meter: &opentelemetry::metrics::Meter, name: impl Into<Cow<'static, str>>) -> Self::Builder<'_> {
                meter.$func(name)
            }
        }
    };
}

/// A counter metric. Alias for `opentelemetry::metrics::Counter<T>`.
///
/// Counter metrics are used to record a value that can only increase.
pub type Counter<T> = opentelemetry::metrics::Counter<T>;

/// A counter metric with a `f64` value.
///
/// Counter metrics are used to record a value that can only increase.
pub type CounterF64 = Counter<f64>;

/// A counter metric with a `u64` value.
///
/// Counter metrics are used to record a value that can only increase.
pub type CounterU64 = Counter<u64>;

impl_collector!(
    CounterF64,
    f64,
    f64_counter,
    opentelemetry::metrics::InstrumentBuilder<'a, CounterF64>
);
impl_collector!(
    CounterU64,
    u64,
    u64_counter,
    opentelemetry::metrics::InstrumentBuilder<'a, CounterU64>
);

/// A gauge metric. Alias for `opentelemetry::metrics::Gauge<T>`.
/// Gauge metrics are used to record a value at the current time, and are not
/// aggregated. If you need to record a value that can be aggregated, use a
/// `Counter` or `UpDownCounter` instead.
pub type Gauge<T> = opentelemetry::metrics::Gauge<T>;

/// A gauge metric with a `f64` value.
///
/// Gauge metrics are used to record a value at the current time, and are not
/// aggregated. If you need to record a value that can be aggregated, use a
/// `Counter` or `UpDownCounter` instead.
pub type GaugeF64 = Gauge<f64>;

/// A gauge metric with a `i64` value.
///
/// Gauge metrics are used to record a value at the current time, and are not
/// aggregated. If you need to record a value that can be aggregated, use a
/// `Counter` or `UpDownCounter` instead.
pub type GaugeI64 = Gauge<i64>;

/// A gauge metric with a `u64` value.
///
/// Gauge metrics are used to record a value at the current time, and are not
/// aggregated. If you need to record a value that can be aggregated, use a
/// `Counter` or `UpDownCounter` instead.
pub type GaugeU64 = Gauge<u64>;

impl_collector!(
    GaugeF64,
    f64,
    f64_gauge,
    opentelemetry::metrics::InstrumentBuilder<'a, GaugeF64>
);
impl_collector!(
    GaugeI64,
    i64,
    i64_gauge,
    opentelemetry::metrics::InstrumentBuilder<'a, GaugeI64>
);
impl_collector!(
    GaugeU64,
    u64,
    u64_gauge,
    opentelemetry::metrics::InstrumentBuilder<'a, GaugeU64>
);

/// A histogram metric. Alias for `opentelemetry::metrics::Histogram<T>`.
///
/// Histograms are used to record a distribution of values.
pub type Histogram<T> = opentelemetry::metrics::Histogram<T>;

/// A histogram metric with a `f64` value.
///
/// Histograms are used to record a distribution of values.
pub type HistogramF64 = Histogram<f64>;

/// A histogram metric with a `u64` value.
///
/// Histograms are used to record a distribution of values.
pub type HistogramU64 = Histogram<u64>;

impl private::Sealed for HistogramF64 {
    type Value = f64;
}

/// Default boundaries for a histogram in Golang.
const DEFAULT_BOUNDARIES: [f64; 11] = [0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0];

impl IsCollector for HistogramF64 {
    type Builder<'a> = opentelemetry::metrics::HistogramBuilder<'a, HistogramF64>;

    fn builder(meter: &opentelemetry::metrics::Meter, name: impl Into<Cow<'static, str>>) -> Self::Builder<'_> {
        meter.f64_histogram(name).with_boundaries(DEFAULT_BOUNDARIES.into())
    }
}

impl private::Sealed for HistogramU64 {
    type Value = u64;
}

impl IsCollector for HistogramU64 {
    type Builder<'a> = opentelemetry::metrics::HistogramBuilder<'a, HistogramU64>;

    fn builder(meter: &opentelemetry::metrics::Meter, name: impl Into<Cow<'static, str>>) -> Self::Builder<'_> {
        meter.u64_histogram(name).with_boundaries(DEFAULT_BOUNDARIES.into())
    }
}

/// A updown counter metric. Alias for
/// `opentelemetry::metrics::UpDownCounter<T>`.
///
/// UpDownCounter like the `Counter` metric, but can also decrement.
pub type UpDownCounter<T> = opentelemetry::metrics::UpDownCounter<T>;

/// A updown counter metric with a `i64` value.
///
/// UpDownCounter like the `Counter` metric, but can also decrement.
pub type UpDownCounterI64 = UpDownCounter<i64>;

/// A updown counter metric with a `f64` value.
///
/// UpDownCounter like the `Counter` metric, but can also decrement.
pub type UpDownCounterF64 = UpDownCounter<f64>;

impl_collector!(
    UpDownCounterI64,
    i64,
    i64_up_down_counter,
    opentelemetry::metrics::InstrumentBuilder<'a, UpDownCounterI64>
);
impl_collector!(
    UpDownCounterF64,
    f64,
    f64_up_down_counter,
    opentelemetry::metrics::InstrumentBuilder<'a, UpDownCounterF64>
);

/// Helper trait to get a value of one for a number type.
/// Used by the macros below to increment and decrement counters.
trait Number {
    const ONE: Self;
}

impl Number for f64 {
    const ONE: Self = 1.0;
}

impl Number for u64 {
    const ONE: Self = 1;
}

impl Number for i64 {
    const ONE: Self = 1;
}

/// A collector is a wrapper around a metric with some attributes.
///
/// Please use the [`#[metrics]`](crate::metrics) macro to create collectors.
#[must_use = "Collectors do nothing by themselves, you must call them"]
pub struct Collector<'a, T: IsCollector> {
    attributes: Vec<KeyValue>,
    collector: &'a T,
}

impl<'a, T: IsCollector> Collector<'a, T> {
    /// Wraps a given collector with the provided attributes.
    ///
    /// This is typically used internally for constructing types
    /// when using the [`#[metrics]`](crate::metrics) module or function attribute.
    pub fn new(attributes: Vec<KeyValue>, collector: &'a T) -> Self {
        Self { attributes, collector }
    }

    /// Returns the inner collector.
    pub fn inner(&self) -> &'a T {
        self.collector
    }
}

macro_rules! impl_counter {
    ($t:ty) => {
        impl<'a> Collector<'a, opentelemetry::metrics::Counter<$t>> {
            /// Increments the counter by one.
            #[inline]
            pub fn incr(&self) {
                self.incr_by(<$t as Number>::ONE);
            }

            /// Increments the counter by the given value.
            pub fn incr_by(&self, value: $t) {
                self.collector.add(value, &self.attributes);
            }
        }
    };
}

impl_counter!(u64);
impl_counter!(f64);

macro_rules! impl_gauge {
    ($t:ty) => {
        impl<'a> Collector<'a, opentelemetry::metrics::Gauge<$t>> {
            /// Sets the value of the gauge.
            pub fn record(&self, value: $t) {
                self.collector.record(value, &self.attributes);
            }
        }
    };
}

impl_gauge!(u64);
impl_gauge!(f64);
impl_gauge!(i64);

macro_rules! impl_histogram {
    ($t:ty) => {
        impl<'a> Collector<'a, opentelemetry::metrics::Histogram<$t>> {
            /// Observes a new value.
            pub fn observe(&self, value: $t) {
                self.collector.record(value, &self.attributes);
            }
        }
    };
}

impl_histogram!(u64);
impl_histogram!(f64);

macro_rules! impl_updowncounter {
    ($t:ty) => {
        impl<'a> Collector<'a, opentelemetry::metrics::UpDownCounter<$t>> {
            /// Increments the counter by one.
            pub fn incr(&self) {
                self.incr_by(<$t as Number>::ONE);
            }

            /// Increments the counter by the given value.
            pub fn incr_by(&self, value: $t) {
                self.collector.add(value, &self.attributes);
            }

            /// Decrements the counter by one.
            pub fn decr(&self) {
                self.decr_by(<$t as Number>::ONE);
            }

            /// Decrements the counter by the given value.
            pub fn decr_by(&self, value: $t) {
                self.collector.add(-value, &self.attributes);
            }
        }
    };
}

impl_updowncounter!(i64);
impl_updowncounter!(f64);

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use std::sync::Arc;

    use opentelemetry::{KeyValue, Value};
    use opentelemetry_sdk::Resource;
    use opentelemetry_sdk::metrics::data::{AggregatedMetrics, MetricData, ResourceMetrics};
    use opentelemetry_sdk::metrics::reader::MetricReader;
    use opentelemetry_sdk::metrics::{ManualReader, ManualReaderBuilder, SdkMeterProvider};

    use crate::HistogramF64;
    use crate::collector::{Collector, IsCollector};

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

        fn temporality(&self, kind: opentelemetry_sdk::metrics::InstrumentKind) -> opentelemetry_sdk::metrics::Temporality {
            self.0.temporality(kind)
        }
    }

    fn setup_reader() -> TestReader {
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
        reader
    }

    fn find_metric<'a>(metrics: &'a ResourceMetrics, name: &str) -> Option<&'a opentelemetry_sdk::metrics::data::Metric> {
        metrics
            .scope_metrics()
            .find(|sm| sm.scope().name() == "scuffle-metrics")
            .and_then(|sm| sm.metrics().find(|m| m.name() == name))
    }

    fn get_data_point_value<'a, T: PartialEq + std::fmt::Debug + Copy + 'a>(
        mut data_points: impl Iterator<Item = &'a opentelemetry_sdk::metrics::data::SumDataPoint<T>>,
        attr_key: &str,
        attr_value: &str,
    ) -> T {
        data_points
            .find(|dp| {
                dp.attributes()
                    .any(|kv| kv.key.as_str() == attr_key && kv.value.as_str() == attr_value)
            })
            .map(|dp| dp.value())
            .expect("Data point not found")
    }

    fn get_histogram_sum<'a>(
        mut data_points: impl Iterator<Item = &'a opentelemetry_sdk::metrics::data::HistogramDataPoint<u64>>,
        attr_key: &str,
        attr_value: &str,
    ) -> u64 {
        data_points
            .find(|dp| {
                dp.attributes()
                    .any(|kv| kv.key.as_str() == attr_key && kv.value.as_str() == attr_value)
            })
            .map(|dp| dp.sum())
            .expect("Histogram data point not found")
    }

    fn get_data_point_value_with_two_attrs<'a, T: PartialEq + std::fmt::Debug + Copy + 'a>(
        mut data_points: impl Iterator<Item = &'a opentelemetry_sdk::metrics::data::SumDataPoint<T>>,
        key1: &str,
        val1: &str,
        key2: &str,
        val2: impl Into<Value>,
    ) -> T {
        let val2 = val2.into();
        data_points
            .find(|dp| {
                dp.attributes().any(|kv| kv.key.as_str() == key1 && kv.value.as_str() == val1)
                    && dp.attributes().any(|kv| kv.key.as_str() == key2 && kv.value == val2)
            })
            .map(|dp| dp.value())
            .expect("Data point not found")
    }

    #[test]
    fn test_counter_metric() {
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

        let reader = setup_reader();
        example::request(example::Kind::Http).incr();
        example::request(example::Kind::Http).incr();
        example::request(example::Kind::Grpc).incr();

        let metrics = reader.read();
        let metric = find_metric(&metrics, "example_request").unwrap();
        assert_eq!(metric.unit(), "requests");

        let AggregatedMetrics::U64(MetricData::Sum(sum)) = metric.data() else {
            unreachable!()
        };
        assert_eq!(sum.data_points().count(), 2);
        assert_eq!(get_data_point_value(sum.data_points(), "kind", "Http"), 2);
        assert_eq!(get_data_point_value(sum.data_points(), "kind", "Grpc"), 1);
    }

    #[test]
    fn test_gauge_metric() {
        #[crate::metrics(crate_path = "crate")]
        mod example {
            use crate::GaugeU64;

            #[metrics(unit = "connections")]
            pub fn current_connections() -> GaugeU64;
        }

        let reader = setup_reader();
        example::current_connections().record(10);
        example::current_connections().record(20);

        let metrics = reader.read();
        let metric = find_metric(&metrics, "example_current_connections").unwrap();
        assert_eq!(metric.unit(), "connections");

        let AggregatedMetrics::U64(MetricData::Gauge(gauge)) = metric.data() else {
            unreachable!()
        };
        assert_eq!(gauge.data_points().count(), 1);
        assert_eq!(gauge.data_points().next().unwrap().value(), 20);
        assert_eq!(gauge.data_points().next().unwrap().attributes().count(), 0);
    }

    #[test]
    fn test_histogram_metric() {
        #[crate::metrics(crate_path = "crate")]
        mod example {
            use crate::{HistogramU64, MetricEnum};

            #[derive(MetricEnum)]
            #[metrics(crate_path = "crate")]
            pub enum Kind {
                Http,
                Grpc,
            }

            #[metrics(unit = "bytes")]
            pub fn data_transfer(kind: Kind) -> HistogramU64;
        }

        let reader = setup_reader();
        example::data_transfer(example::Kind::Http).observe(100);
        example::data_transfer(example::Kind::Http).observe(200);
        example::data_transfer(example::Kind::Grpc).observe(150);

        let metrics = reader.read();
        let metric = find_metric(&metrics, "example_data_transfer").unwrap();
        assert_eq!(metric.unit(), "bytes");

        let AggregatedMetrics::U64(MetricData::Histogram(histogram)) = metric.data() else {
            unreachable!()
        };

        assert_eq!(histogram.data_points().count(), 2);
        assert_eq!(get_histogram_sum(histogram.data_points(), "kind", "Http"), 300);
        assert_eq!(get_histogram_sum(histogram.data_points(), "kind", "Grpc"), 150);
    }

    #[test]
    fn test_updowncounter_metric() {
        #[crate::metrics(crate_path = "crate")]
        mod example {
            use crate::{MetricEnum, UpDownCounterI64};

            #[derive(MetricEnum)]
            #[metrics(crate_path = "crate")]
            pub enum Kind {
                Http,
                Grpc,
            }

            #[metrics(unit = "requests")]
            pub fn active_requests(kind: Kind) -> UpDownCounterI64;
        }

        let reader = setup_reader();
        example::active_requests(example::Kind::Http).incr();
        example::active_requests(example::Kind::Http).incr();
        example::active_requests(example::Kind::Http).decr();
        example::active_requests(example::Kind::Grpc).incr();

        let metrics = reader.read();
        let metric = find_metric(&metrics, "example_active_requests").unwrap();
        assert_eq!(metric.unit(), "requests");

        let AggregatedMetrics::I64(MetricData::Sum(sum)) = metric.data() else {
            unreachable!()
        };

        assert_eq!(sum.data_points().count(), 2);
        assert_eq!(get_data_point_value(sum.data_points(), "kind", "Http"), 1);
        assert_eq!(get_data_point_value(sum.data_points(), "kind", "Grpc"), 1);
    }

    #[test]
    fn test_metric_with_multiple_attributes() {
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
            pub fn request_with_status(kind: Kind, status: u32) -> CounterU64;
        }

        let reader = setup_reader();
        example::request_with_status(example::Kind::Http, 200).incr();
        example::request_with_status(example::Kind::Http, 404).incr();
        example::request_with_status(example::Kind::Grpc, 200).incr();

        let metrics = reader.read();
        let metric = find_metric(&metrics, "example_request_with_status").unwrap();
        assert_eq!(metric.unit(), "requests");

        let AggregatedMetrics::U64(MetricData::Sum(sum)) = metric.data() else {
            unreachable!()
        };
        assert_eq!(sum.data_points().count(), 3);
        assert_eq!(
            get_data_point_value_with_two_attrs(sum.data_points(), "kind", "Http", "status", 200),
            1
        );
        assert_eq!(
            get_data_point_value_with_two_attrs(sum.data_points(), "kind", "Http", "status", 404),
            1
        );
        assert_eq!(
            get_data_point_value_with_two_attrs(sum.data_points(), "kind", "Grpc", "status", 200),
            1
        );
    }

    #[test]
    fn test_metric_with_string_attribute() {
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
            pub fn request_with_method(kind: Kind, method: &str) -> CounterU64;
        }

        let reader = setup_reader();
        example::request_with_method(example::Kind::Http, "GET").incr();
        example::request_with_method(example::Kind::Http, "POST").incr();
        example::request_with_method(example::Kind::Grpc, "GET").incr();

        let metrics = reader.read();
        let metric = find_metric(&metrics, "example_request_with_method").unwrap();
        assert_eq!(metric.unit(), "requests");

        let AggregatedMetrics::U64(MetricData::Sum(sum)) = metric.data() else {
            unreachable!()
        };
        assert_eq!(sum.data_points().count(), 3);
        assert_eq!(
            get_data_point_value_with_two_attrs(sum.data_points(), "kind", "Http", "method", "GET"),
            1
        );
        assert_eq!(
            get_data_point_value_with_two_attrs(sum.data_points(), "kind", "Http", "method", "POST"),
            1
        );
        assert_eq!(
            get_data_point_value_with_two_attrs(sum.data_points(), "kind", "Grpc", "method", "GET"),
            1
        );
    }

    #[test]
    fn test_metric_with_no_attributes() {
        #[crate::metrics(crate_path = "crate")]
        mod example {
            use crate::CounterU64;

            #[metrics(unit = "events")]
            pub fn total_events() -> CounterU64;
        }

        let reader = setup_reader();
        example::total_events().incr();
        example::total_events().incr();

        let metrics = reader.read();
        let metric = find_metric(&metrics, "example_total_events").unwrap();
        assert_eq!(metric.unit(), "events");

        let AggregatedMetrics::U64(MetricData::Sum(sum)) = metric.data() else {
            unreachable!()
        };
        assert_eq!(sum.data_points().count(), 1);
        assert_eq!(sum.data_points().next().unwrap().value(), 2);
        assert_eq!(sum.data_points().next().unwrap().attributes().count(), 0);
    }

    #[test]
    fn test_metric_with_zero_values() {
        #[crate::metrics(crate_path = "crate")]
        mod example {
            use crate::GaugeU64;

            #[metrics(unit = "connections")]
            pub fn current_connections() -> GaugeU64;
        }

        let reader = setup_reader();
        example::current_connections().record(0);

        let metrics = reader.read();
        let metric = find_metric(&metrics, "example_current_connections").unwrap();
        assert_eq!(metric.unit(), "connections");

        let AggregatedMetrics::U64(MetricData::Gauge(gauge)) = metric.data() else {
            unreachable!()
        };
        assert_eq!(gauge.data_points().count(), 1);
        assert_eq!(gauge.data_points().next().unwrap().value(), 0);
        assert_eq!(gauge.data_points().next().unwrap().attributes().count(), 0);
    }

    #[test]
    fn test_metric_with_negative_increments() {
        #[crate::metrics(crate_path = "crate")]
        mod example {
            use crate::{MetricEnum, UpDownCounterI64};

            #[derive(MetricEnum)]
            #[metrics(crate_path = "crate")]
            pub enum Kind {
                Http,
                Grpc,
            }

            #[metrics(unit = "requests")]
            pub fn active_requests(kind: Kind) -> UpDownCounterI64;
        }

        let reader = setup_reader();
        example::active_requests(example::Kind::Http).incr();
        example::active_requests(example::Kind::Http).decr();
        example::active_requests(example::Kind::Http).decr();

        let metrics = reader.read();
        let metric = find_metric(&metrics, "example_active_requests").unwrap();
        assert_eq!(metric.unit(), "requests");

        let AggregatedMetrics::I64(MetricData::Sum(sum)) = metric.data() else {
            unreachable!()
        };
        assert_eq!(sum.data_points().count(), 1);
        assert_eq!(get_data_point_value(sum.data_points(), "kind", "Http"), -1);
    }

    #[test]
    fn test_histogram_f64_builder() {
        let reader = setup_reader();
        let meter = opentelemetry::global::meter("scuffle-metrics");
        let name = "test_histogram_f64";

        let builder = HistogramF64::builder(&meter, name);
        let histogram = builder.build();

        histogram.record(1.5, &[]);

        let metrics = reader.read();
        let metric = find_metric(&metrics, name).expect("histogram metric not found");

        assert_eq!(metric.name(), name);
        assert_eq!(metric.unit(), "");

        let AggregatedMetrics::F64(MetricData::Histogram(histogram_data)) = metric.data() else {
            unreachable!()
        };

        assert_eq!(histogram_data.data_points().count(), 1);
        assert_eq!(histogram_data.data_points().next().unwrap().sum(), 1.5);
        assert_eq!(histogram_data.data_points().next().unwrap().attributes().count(), 0);
    }

    #[test]
    fn test_collector_inner() {
        let meter = opentelemetry::global::meter("test_meter");
        let histogram = HistogramF64::builder(&meter, "inner_test_histogram").build();

        let attributes = vec![KeyValue::new("key", "value")];
        let collector = Collector::new(attributes.clone(), &histogram);

        assert_eq!(collector.inner() as *const HistogramF64, &histogram as *const HistogramF64);
    }
}
