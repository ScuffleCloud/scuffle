use std::borrow::Cow;
use std::sync::Arc;

use opentelemetry::{InstrumentationScope, KeyValue, otel_error, otel_warn};
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::metrics::data::{Gauge, Histogram, ResourceMetrics, Sum};
use opentelemetry_sdk::metrics::reader::MetricReader;
use opentelemetry_sdk::metrics::{ManualReader, ManualReaderBuilder};
use prometheus_client::encoding::{EncodeCounterValue, EncodeGaugeValue, NoLabelSet};
use prometheus_client::metrics::MetricType;
use prometheus_client::registry::Unit;

/// A Prometheus exporter for OpenTelemetry metrics.
///
/// Responsible for encoding OpenTelemetry metrics into Prometheus format.
/// The exporter implements the
/// [`opentelemetry_sdk::metrics::reader::MetricReader`](https://docs.rs/opentelemetry_sdk/0.27.0/opentelemetry_sdk/metrics/reader/trait.MetricReader.html)
/// trait and therefore can be passed to a
/// [`opentelemetry_sdk::metrics::SdkMeterProvider`](https://docs.rs/opentelemetry_sdk/0.27.0/opentelemetry_sdk/metrics/struct.SdkMeterProvider.html).
///
/// Use [`collector`](PrometheusExporter::collector) to get a
/// [`prometheus_client::collector::Collector`](https://docs.rs/prometheus-client/0.22.3/prometheus_client/collector/trait.Collector.html)
/// that can be registered with a
/// [`prometheus_client::registry::Registry`](https://docs.rs/prometheus-client/0.22.3/prometheus_client/registry/struct.Registry.html)
/// to provide metrics to Prometheus.
#[derive(Debug, Clone)]
pub struct PrometheusExporter {
    reader: Arc<ManualReader>,
    prometheus_full_utf8: bool,
}

impl PrometheusExporter {
    /// Returns a new [`PrometheusExporterBuilder`] to configure a [`PrometheusExporter`].
    pub fn builder() -> PrometheusExporterBuilder {
        PrometheusExporterBuilder::default()
    }

    /// Returns a [`prometheus_client::collector::Collector`] that can be registered
    /// with a [`prometheus_client::registry::Registry`] to provide metrics to Prometheus.
    pub fn collector(&self) -> Box<dyn prometheus_client::collector::Collector> {
        Box::new(self.clone())
    }
}

impl MetricReader for PrometheusExporter {
    fn register_pipeline(&self, pipeline: std::sync::Weak<opentelemetry_sdk::metrics::Pipeline>) {
        self.reader.register_pipeline(pipeline)
    }

    fn collect(
        &self,
        rm: &mut opentelemetry_sdk::metrics::data::ResourceMetrics,
    ) -> opentelemetry_sdk::metrics::MetricResult<()> {
        self.reader.collect(rm)
    }

    fn force_flush(&self) -> opentelemetry_sdk::error::OTelSdkResult {
        self.reader.force_flush()
    }

    fn shutdown(&self) -> opentelemetry_sdk::error::OTelSdkResult {
        self.reader.shutdown()
    }

    fn temporality(&self, kind: opentelemetry_sdk::metrics::InstrumentKind) -> opentelemetry_sdk::metrics::Temporality {
        self.reader.temporality(kind)
    }
}

/// Builder for [`PrometheusExporter`].
#[derive(Default)]
pub struct PrometheusExporterBuilder {
    reader: ManualReaderBuilder,
    prometheus_full_utf8: bool,
}

impl PrometheusExporterBuilder {
    /// Set the reader temporality.
    pub fn with_temporality(mut self, temporality: opentelemetry_sdk::metrics::Temporality) -> Self {
        self.reader = self.reader.with_temporality(temporality);
        self
    }

    /// Allow full UTF-8 labels in Prometheus.
    ///
    /// This is disabled by default however if you are using a newer version of
    /// Prometheus that supports full UTF-8 labels you may enable this feature.
    pub fn with_prometheus_full_utf8(mut self, prometheus_full_utf8: bool) -> Self {
        self.prometheus_full_utf8 = prometheus_full_utf8;
        self
    }

    /// Build the [`PrometheusExporter`].
    pub fn build(self) -> PrometheusExporter {
        PrometheusExporter {
            reader: Arc::new(self.reader.build()),
            prometheus_full_utf8: self.prometheus_full_utf8,
        }
    }
}

/// Returns a new [`PrometheusExporterBuilder`] to configure a [`PrometheusExporter`].
pub fn exporter() -> PrometheusExporterBuilder {
    PrometheusExporter::builder()
}

#[derive(Debug, Clone, Copy)]
enum RawNumber {
    U64(u64),
    I64(i64),
    F64(f64),
}

impl RawNumber {
    fn as_f64(&self) -> f64 {
        match *self {
            RawNumber::U64(value) => value as f64,
            RawNumber::I64(value) => value as f64,
            RawNumber::F64(value) => value,
        }
    }
}

impl EncodeGaugeValue for RawNumber {
    fn encode(&self, encoder: &mut prometheus_client::encoding::GaugeValueEncoder) -> Result<(), std::fmt::Error> {
        match *self {
            RawNumber::U64(value) => EncodeGaugeValue::encode(&(value as i64), encoder),
            RawNumber::I64(value) => EncodeGaugeValue::encode(&value, encoder),
            RawNumber::F64(value) => EncodeGaugeValue::encode(&value, encoder),
        }
    }
}

impl EncodeCounterValue for RawNumber {
    fn encode(&self, encoder: &mut prometheus_client::encoding::CounterValueEncoder) -> Result<(), std::fmt::Error> {
        match *self {
            RawNumber::U64(value) => EncodeCounterValue::encode(&value, encoder),
            RawNumber::I64(value) => EncodeCounterValue::encode(&(value as f64), encoder),
            RawNumber::F64(value) => EncodeCounterValue::encode(&value, encoder),
        }
    }
}

macro_rules! impl_raw_number {
    ($t:ty, $variant:ident) => {
        impl From<$t> for RawNumber {
            fn from(value: $t) -> Self {
                RawNumber::$variant(value)
            }
        }
    };
}

impl_raw_number!(u64, U64);
impl_raw_number!(i64, I64);
impl_raw_number!(f64, F64);

enum KnownMetricT<'a, T> {
    Gauge(&'a Gauge<T>),
    Sum(&'a Sum<T>),
    Histogram(&'a Histogram<T>),
}

impl<'a, T: 'static> KnownMetricT<'a, T>
where
    RawNumber: From<T>,
    T: Copy,
{
    fn from_any(any: &'a dyn std::any::Any) -> Option<Self> {
        if let Some(gauge) = any.downcast_ref::<Gauge<T>>() {
            Some(KnownMetricT::Gauge(gauge))
        } else if let Some(sum) = any.downcast_ref::<Sum<T>>() {
            Some(KnownMetricT::Sum(sum))
        } else {
            any.downcast_ref::<Histogram<T>>()
                .map(|histogram| KnownMetricT::Histogram(histogram))
        }
    }

    fn metric_type(&self) -> MetricType {
        match self {
            KnownMetricT::Gauge(_) => MetricType::Gauge,
            KnownMetricT::Sum(sum) => {
                if sum.is_monotonic {
                    MetricType::Counter
                } else {
                    MetricType::Gauge
                }
            }
            KnownMetricT::Histogram(_) => MetricType::Histogram,
        }
    }

    fn encode(
        &self,
        mut encoder: prometheus_client::encoding::MetricEncoder,
        labels: KeyValueEncoder<'a>,
    ) -> Result<(), std::fmt::Error> {
        match self {
            KnownMetricT::Gauge(gauge) => {
                for data_point in &gauge.data_points {
                    let number = RawNumber::from(data_point.value);
                    encoder
                        .encode_family(&labels.with_attrs(Some(&data_point.attributes)))?
                        .encode_gauge(&number)?;
                }
            }
            KnownMetricT::Sum(sum) => {
                for data_point in &sum.data_points {
                    let number = RawNumber::from(data_point.value);
                    let attrs = labels.with_attrs(Some(&data_point.attributes));
                    let mut encoder = encoder.encode_family(&attrs)?;

                    if sum.is_monotonic {
                        // TODO(troy): Exemplar support
                        encoder.encode_counter::<NoLabelSet, _, f64>(&number, None)?;
                    } else {
                        encoder.encode_gauge(&number)?;
                    }
                }
            }
            KnownMetricT::Histogram(histogram) => {
                for data_point in &histogram.data_points {
                    let attrs = labels.with_attrs(Some(&data_point.attributes));
                    let mut encoder = encoder.encode_family(&attrs)?;

                    let sum = RawNumber::from(data_point.sum);

                    let buckets = data_point
                        .bounds
                        .iter()
                        .copied()
                        .zip(data_point.bucket_counts.iter().copied())
                        .collect::<Vec<_>>();

                    encoder.encode_histogram::<NoLabelSet>(sum.as_f64(), data_point.count, &buckets, None)?;
                }
            }
        }

        Ok(())
    }
}

enum KnownMetric<'a> {
    U64(KnownMetricT<'a, u64>),
    I64(KnownMetricT<'a, i64>),
    F64(KnownMetricT<'a, f64>),
}

impl<'a> KnownMetric<'a> {
    fn from_any(any: &'a dyn std::any::Any) -> Option<Self> {
        macro_rules! try_decode {
            ($t:ty, $variant:ident) => {
                if let Some(metric) = KnownMetricT::<$t>::from_any(any) {
                    return Some(KnownMetric::$variant(metric));
                }
            };
        }

        try_decode!(u64, U64);
        try_decode!(i64, I64);
        try_decode!(f64, F64);

        None
    }

    fn metric_type(&self) -> MetricType {
        match self {
            KnownMetric::U64(metric) => metric.metric_type(),
            KnownMetric::I64(metric) => metric.metric_type(),
            KnownMetric::F64(metric) => metric.metric_type(),
        }
    }

    fn encode(
        &self,
        encoder: prometheus_client::encoding::MetricEncoder,
        labels: KeyValueEncoder<'a>,
    ) -> Result<(), std::fmt::Error> {
        match self {
            KnownMetric::U64(metric) => metric.encode(encoder, labels),
            KnownMetric::I64(metric) => metric.encode(encoder, labels),
            KnownMetric::F64(metric) => metric.encode(encoder, labels),
        }
    }
}

impl prometheus_client::collector::Collector for PrometheusExporter {
    fn encode(&self, mut encoder: prometheus_client::encoding::DescriptorEncoder) -> Result<(), std::fmt::Error> {
        let mut metrics = ResourceMetrics {
            resource: Resource::builder_empty().build(),
            scope_metrics: vec![],
        };

        if let Err(err) = self.reader.collect(&mut metrics) {
            otel_error!(name: "prometheus_collector_collect_error", error = err.to_string());
            return Err(std::fmt::Error);
        }

        let labels = KeyValueEncoder::new(self.prometheus_full_utf8);

        encoder
            .encode_descriptor("target", "Information about the target", None, MetricType::Info)?
            .encode_info(&labels.with_resource(Some(&metrics.resource)))?;

        for scope_metrics in &metrics.scope_metrics {
            for metric in &scope_metrics.metrics {
                let Some(known_metric) = KnownMetric::from_any(metric.data.as_any()) else {
                    otel_warn!(name: "prometheus_collector_unknown_metric_type", metric_name = metric.name.as_ref());
                    continue;
                };

                let unit = if metric.unit.is_empty() {
                    None
                } else {
                    Some(Unit::Other(metric.unit.to_string()))
                };

                known_metric.encode(
                    encoder.encode_descriptor(
                        &metric.name,
                        &metric.description,
                        unit.as_ref(),
                        known_metric.metric_type(),
                    )?,
                    labels.with_scope(Some(&scope_metrics.scope)),
                )?;
            }
        }

        Ok(())
    }
}

fn scope_to_iter(scope: &InstrumentationScope) -> impl Iterator<Item = (&str, Cow<'_, str>)> {
    [
        ("otel.scope.name", Some(Cow::Borrowed(scope.name()))),
        ("otel.scope.version", scope.version().map(Cow::Borrowed)),
        ("otel.scope.schema_url", scope.schema_url().map(Cow::Borrowed)),
    ]
    .into_iter()
    .chain(scope.attributes().map(|kv| (kv.key.as_str(), Some(kv.value.as_str()))))
    .filter_map(|(key, value)| value.map(|v| (key, v)))
}

#[derive(Debug, Clone, Copy)]
struct KeyValueEncoder<'a> {
    resource: Option<&'a Resource>,
    scope: Option<&'a InstrumentationScope>,
    attrs: Option<&'a [KeyValue]>,
    prometheus_full_utf8: bool,
}

impl<'a> KeyValueEncoder<'a> {
    fn new(prometheus_full_utf8: bool) -> Self {
        Self {
            resource: None,
            scope: None,
            attrs: None,
            prometheus_full_utf8,
        }
    }

    fn with_resource(self, resource: Option<&'a Resource>) -> Self {
        Self { resource, ..self }
    }

    fn with_scope(self, scope: Option<&'a InstrumentationScope>) -> Self {
        Self { scope, ..self }
    }

    fn with_attrs(self, attrs: Option<&'a [KeyValue]>) -> Self {
        Self { attrs, ..self }
    }
}

fn escape_key(s: &str) -> Cow<'_, str> {
    // prefix chars to add in case name starts with number
    let mut prefix = "";

    // Find first invalid char
    if let Some((replace_idx, _)) = s.char_indices().find(|(i, c)| {
        if *i == 0 && c.is_ascii_digit() {
            // first char is number, add prefix and replace reset of chars
            prefix = "_";
            true
        } else {
            // keep checking
            !c.is_alphanumeric() && *c != '_' && *c != ':'
        }
    }) {
        // up to `replace_idx` have been validated, convert the rest
        let (valid, rest) = s.split_at(replace_idx);
        Cow::Owned(
            prefix
                .chars()
                .chain(valid.chars())
                .chain(rest.chars().map(|c| {
                    if c.is_ascii_alphanumeric() || c == '_' || c == ':' {
                        c
                    } else {
                        '_'
                    }
                }))
                .collect(),
        )
    } else {
        Cow::Borrowed(s) // no invalid chars found, return existing
    }
}

impl prometheus_client::encoding::EncodeLabelSet for KeyValueEncoder<'_> {
    fn encode(&self, mut encoder: prometheus_client::encoding::LabelSetEncoder) -> Result<(), std::fmt::Error> {
        use std::fmt::Write;

        fn write_kv(
            encoder: &mut prometheus_client::encoding::LabelSetEncoder,
            key: &str,
            value: &str,
            prometheus_full_utf8: bool,
        ) -> Result<(), std::fmt::Error> {
            let mut label = encoder.encode_label();
            let mut key_encoder = label.encode_label_key()?;
            if prometheus_full_utf8 {
                // TODO(troy): I am not sure if this is correct.
                // See: https://github.com/prometheus/client_rust/issues/251
                write!(&mut key_encoder, "{key}")?;
            } else {
                write!(&mut key_encoder, "{}", escape_key(key))?;
            }

            let mut value_encoder = key_encoder.encode_label_value()?;
            write!(&mut value_encoder, "{value}")?;

            value_encoder.finish()
        }

        if let Some(resource) = self.resource {
            for (key, value) in resource.iter() {
                write_kv(&mut encoder, key.as_str(), value.as_str().as_ref(), self.prometheus_full_utf8)?;
            }
        }

        if let Some(scope) = self.scope {
            for (key, value) in scope_to_iter(scope) {
                write_kv(&mut encoder, key, value.as_ref(), self.prometheus_full_utf8)?;
            }
        }

        if let Some(attrs) = self.attrs {
            for kv in attrs {
                write_kv(
                    &mut encoder,
                    kv.key.as_str(),
                    kv.value.as_str().as_ref(),
                    self.prometheus_full_utf8,
                )?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use opentelemetry::KeyValue;
    use opentelemetry::metrics::MeterProvider;
    use opentelemetry_sdk::Resource;
    use opentelemetry_sdk::metrics::SdkMeterProvider;
    use prometheus_client::registry::Registry;

    use super::*;

    fn setup_prometheus_exporter(
        temporality: opentelemetry_sdk::metrics::Temporality,
        full_utf8: bool,
    ) -> (PrometheusExporter, Registry) {
        let exporter = PrometheusExporter::builder()
            .with_temporality(temporality)
            .with_prometheus_full_utf8(full_utf8)
            .build();
        let mut registry = Registry::default();
        registry.register_collector(exporter.collector());
        (exporter, registry)
    }

    fn collect_and_encode(registry: &Registry) -> String {
        let mut buffer = String::new();
        prometheus_client::encoding::text::encode(&mut buffer, registry).unwrap();
        buffer
    }

    #[test]
    fn test_prometheus_collect() {
        let (exporter, registry) = setup_prometheus_exporter(opentelemetry_sdk::metrics::Temporality::Cumulative, false);
        let provider = SdkMeterProvider::builder()
            .with_reader(exporter.clone())
            .with_resource(
                Resource::builder()
                    .with_attributes(vec![KeyValue::new("service.name", "test_service")])
                    .build(),
            )
            .build();
        opentelemetry::global::set_meter_provider(provider.clone());

        let meter = provider.meter("test_meter");
        let counter = meter.u64_counter("test_counter").build();
        counter.add(1, &[KeyValue::new("key", "value")]);

        let encoded = collect_and_encode(&registry);

        assert!(encoded.contains("test_counter"));
        assert!(encoded.contains(r#"key="value""#));
        assert!(encoded.contains(r#"test_counter_total{otel_scope_name="test_meter",key="value"} 1"#));
    }

    #[test]
    fn test_prometheus_temporality() {
        let exporter = PrometheusExporter::builder()
            .with_temporality(opentelemetry_sdk::metrics::Temporality::Delta)
            .build();

        let temporality = exporter.temporality(opentelemetry_sdk::metrics::InstrumentKind::Counter);

        assert_eq!(temporality, opentelemetry_sdk::metrics::Temporality::Delta);
    }

    #[test]
    fn test_prometheus_full_utf8() {
        let (exporter, registry) = setup_prometheus_exporter(opentelemetry_sdk::metrics::Temporality::Cumulative, true);
        let provider = SdkMeterProvider::builder()
            .with_reader(exporter.clone())
            .with_resource(
                Resource::builder()
                    .with_attributes(vec![KeyValue::new("service.name", "test_service")])
                    .build(),
            )
            .build();
        opentelemetry::global::set_meter_provider(provider.clone());

        let meter = provider.meter("test_meter");
        let counter = meter.u64_counter("test_counter").build();
        counter.add(1, &[KeyValue::new("key_😊", "value_😊")]);

        let encoded = collect_and_encode(&registry);

        assert!(encoded.contains(r#"key_😊="value_😊""#));
    }

    #[test]
    fn test_raw_number_as_f64() {
        assert_eq!(RawNumber::U64(42).as_f64(), 42.0);
        assert_eq!(RawNumber::I64(-42).as_f64(), -42.0);
        assert_eq!(RawNumber::F64(5.44).as_f64(), 5.44);
    }

    #[test]
    fn test_known_metric_t_from_any() {
        let time = std::time::SystemTime::now();
        let gauge = Gauge::<u64> {
            data_points: vec![],
            start_time: Some(time - std::time::Duration::from_secs(10)),
            time,
        };
        let sum = Sum::<u64> {
            data_points: vec![],
            is_monotonic: true,
            start_time: time - std::time::Duration::from_secs(10),
            time,
            temporality: opentelemetry_sdk::metrics::Temporality::Cumulative,
        };
        let histogram = Histogram::<u64> {
            data_points: vec![],
            start_time: time - std::time::Duration::from_secs(10),
            time,
            temporality: opentelemetry_sdk::metrics::Temporality::Cumulative,
        };

        assert!(matches!(KnownMetricT::<u64>::from_any(&gauge), Some(KnownMetricT::Gauge(_))));
        assert!(matches!(KnownMetricT::<u64>::from_any(&sum), Some(KnownMetricT::Sum(_))));
        assert!(matches!(
            KnownMetricT::<u64>::from_any(&histogram),
            Some(KnownMetricT::Histogram(_))
        ));
    }

    #[test]
    fn test_known_metric_t_metric_type() {
        let time = std::time::SystemTime::now();
        let gauge = Gauge::<u64> {
            data_points: vec![],
            start_time: Some(time - std::time::Duration::from_secs(10)),
            time,
        };
        let gauge = KnownMetricT::Gauge(&gauge);
        matches!(gauge.metric_type(), MetricType::Gauge);

        let sum = Sum::<u64> {
            data_points: vec![],
            is_monotonic: true,
            start_time: time - std::time::Duration::from_secs(10),
            time,
            temporality: opentelemetry_sdk::metrics::Temporality::Cumulative,
        };
        let sum_monotonic = KnownMetricT::Sum(&sum);
        matches!(sum_monotonic.metric_type(), MetricType::Counter);

        let sum = Sum::<u64> {
            data_points: vec![],
            is_monotonic: false,
            start_time: time - std::time::Duration::from_secs(10),
            time,
            temporality: opentelemetry_sdk::metrics::Temporality::Cumulative,
        };
        let sum_non_monotonic = KnownMetricT::Sum(&sum);
        matches!(sum_non_monotonic.metric_type(), MetricType::Gauge);

        let histogram = Histogram::<u64> {
            data_points: vec![],
            start_time: time - std::time::Duration::from_secs(10),
            time,
            temporality: opentelemetry_sdk::metrics::Temporality::Cumulative,
        };
        let histogram = KnownMetricT::Histogram(&histogram);
        matches!(histogram.metric_type(), MetricType::Histogram);
    }

    #[test]
    fn test_known_metric_t_encode() {
        let (exporter, registry) = setup_prometheus_exporter(opentelemetry_sdk::metrics::Temporality::Cumulative, false);
        let provider = SdkMeterProvider::builder().with_reader(exporter.clone()).build();
        let meter = provider.meter("test_meter");

        let gauge_u64 = meter.u64_gauge("test_u64_gauge").build();
        gauge_u64.record(42, &[KeyValue::new("key", "value")]);

        let encoded = collect_and_encode(&registry);
        assert!(encoded.contains(r#"test_u64_gauge{otel_scope_name="test_meter",key="value"} 42"#));

        let counter_i64_sum = meter.i64_up_down_counter("test_i64_counter").build();
        counter_i64_sum.add(-42, &[KeyValue::new("key", "value")]);

        let encoded = collect_and_encode(&registry);
        assert!(encoded.contains(r#"test_i64_counter{otel_scope_name="test_meter",key="value"} -42"#));
    }

    #[test]
    fn test_known_metric_from_any() {
        let time = std::time::SystemTime::now();
        let gauge_u64 = Gauge::<u64> {
            data_points: vec![],
            start_time: Some(time),
            time,
        };
        let sum_i64 = Sum::<i64> {
            data_points: vec![],
            is_monotonic: true,
            start_time: time,
            time,
            temporality: opentelemetry_sdk::metrics::Temporality::Cumulative,
        };
        let histogram_f64 = Histogram::<f64> {
            data_points: vec![],
            start_time: time,
            time,
            temporality: opentelemetry_sdk::metrics::Temporality::Cumulative,
        };

        assert!(matches!(
            KnownMetric::from_any(&gauge_u64),
            Some(KnownMetric::U64(KnownMetricT::Gauge(_)))
        ));
        assert!(matches!(
            KnownMetric::from_any(&sum_i64),
            Some(KnownMetric::I64(KnownMetricT::Sum(_)))
        ));
        assert!(matches!(
            KnownMetric::from_any(&histogram_f64),
            Some(KnownMetric::F64(KnownMetricT::Histogram(_)))
        ));
        assert!(KnownMetric::from_any(&true).is_none());
    }

    #[test]
    fn test_known_metric_metric_type() {
        let time = std::time::SystemTime::now();
        let gauge = Gauge::<u64> {
            data_points: vec![],
            start_time: Some(time),
            time,
        };
        let metric = KnownMetric::U64(KnownMetricT::Gauge(&gauge));
        assert!(matches!(metric.metric_type(), MetricType::Gauge));

        let sum_mono = Sum::<i64> {
            data_points: vec![],
            is_monotonic: true,
            start_time: time,
            time,
            temporality: opentelemetry_sdk::metrics::Temporality::Cumulative,
        };
        let metric = KnownMetric::I64(KnownMetricT::Sum(&sum_mono));
        assert!(matches!(metric.metric_type(), MetricType::Counter));

        let sum_non_mono = Sum::<f64> {
            data_points: vec![],
            is_monotonic: false,
            start_time: time,
            time,
            temporality: opentelemetry_sdk::metrics::Temporality::Cumulative,
        };
        let metric = KnownMetric::F64(KnownMetricT::Sum(&sum_non_mono));
        assert!(matches!(metric.metric_type(), MetricType::Gauge));
    }

    #[test]
    fn test_known_metric_encode() {
        let (exporter, registry) = setup_prometheus_exporter(opentelemetry_sdk::metrics::Temporality::Cumulative, false);
        let provider = SdkMeterProvider::builder().with_reader(exporter.clone()).build();
        let meter = provider.meter("test_meter");

        meter
            .f64_counter("test_f64_counter")
            .build()
            .add(1.0, &[KeyValue::new("key", "value")]);
        assert!(
            collect_and_encode(&registry).contains(r#"test_f64_counter_total{otel_scope_name="test_meter",key="value"} 1"#)
        );
        meter
            .u64_counter("test_u64_counter")
            .build()
            .add(1, &[KeyValue::new("key", "value")]);
        assert!(
            collect_and_encode(&registry).contains(r#"test_u64_counter_total{otel_scope_name="test_meter",key="value"} 1"#)
        );
        meter
            .f64_up_down_counter("test_f64_up_down_counter")
            .build()
            .add(1.0, &[KeyValue::new("key", "value")]);
        assert!(
            collect_and_encode(&registry)
                .contains(r#"test_f64_up_down_counter{otel_scope_name="test_meter",key="value"} 1"#)
        );
        meter
            .i64_up_down_counter("test_i64_up_down_counter")
            .build()
            .add(-1, &[KeyValue::new("key", "value")]);
        assert!(
            collect_and_encode(&registry)
                .contains(r#"test_i64_up_down_counter{otel_scope_name="test_meter",key="value"} -1"#)
        );

        meter
            .f64_gauge("test_f64_gauge")
            .build()
            .record(1.0, &[KeyValue::new("key", "value")]);
        assert!(collect_and_encode(&registry).contains(r#"test_f64_gauge{otel_scope_name="test_meter",key="value"} 1"#));
        meter
            .i64_gauge("test_i64_gauge")
            .build()
            .record(-1, &[KeyValue::new("key", "value")]);
        assert!(collect_and_encode(&registry).contains(r#"test_i64_gauge{otel_scope_name="test_meter",key="value"} -1"#));
        meter
            .u64_gauge("test_u64_gauge")
            .build()
            .record(1, &[KeyValue::new("key", "value")]);
        assert!(collect_and_encode(&registry).contains(r#"test_u64_gauge{otel_scope_name="test_meter",key="value"} 1"#));

        meter
            .f64_histogram("test_f64_histogram")
            .build()
            .record(1.0, &[KeyValue::new("key", "value")]);
        assert!(
            collect_and_encode(&registry).contains(r#"test_f64_histogram_sum{otel_scope_name="test_meter",key="value"} 1"#)
        );
        meter
            .u64_histogram("test_u64_histogram")
            .build()
            .record(1, &[KeyValue::new("key", "value")]);
        assert!(
            collect_and_encode(&registry).contains(r#"test_u64_histogram_sum{otel_scope_name="test_meter",key="value"} 1"#)
        );
    }

    #[test]
    fn test_prometheus_collect_histogram() {
        let (exporter, registry) = setup_prometheus_exporter(opentelemetry_sdk::metrics::Temporality::Cumulative, false);
        let provider = SdkMeterProvider::builder().with_reader(exporter.clone()).build();
        let meter = provider.meter("test_meter");
        let histogram = meter
            .u64_histogram("test_histogram")
            .with_boundaries(vec![5.0, 10.0, 20.0])
            .build();
        histogram.record(3, &[KeyValue::new("key", "value")]);
        histogram.record(7, &[KeyValue::new("key", "value")]);
        histogram.record(12, &[KeyValue::new("key", "value")]);
        histogram.record(25, &[KeyValue::new("key", "value")]);

        let mut metrics = ResourceMetrics {
            scope_metrics: vec![],
            resource: Resource::builder_empty().build(),
        };
        exporter.collect(&mut metrics).unwrap();

        let scope_metrics = metrics.scope_metrics.first().expect("scope metrics should be present");
        let metric = scope_metrics
            .metrics
            .iter()
            .find(|m| m.name == "test_histogram")
            .expect("histogram metric should be present");
        let histogram_data = metric
            .data
            .as_any()
            .downcast_ref::<Histogram<u64>>()
            .expect("metric data should be a histogram");

        let data_point = histogram_data.data_points.first().expect("data point should be present");
        assert_eq!(data_point.sum, 47, "sum should be 3 + 7 + 12 + 25 = 47");
        assert_eq!(data_point.count, 4, "count should be 4");
        assert_eq!(
            data_point.bucket_counts,
            vec![1, 1, 1, 1],
            "each value should fall into a separate bucket"
        );
        assert_eq!(
            data_point.bounds,
            vec![5.0, 10.0, 20.0],
            "boundaries should match the defined ones"
        );

        let encoded = collect_and_encode(&registry);
        assert!(encoded.contains(r#"test_histogram_sum{otel_scope_name="test_meter",key="value"} 47"#));
    }

    #[test]
    fn test_non_monotonic_sum_as_gauge() {
        let (exporter, registry) = setup_prometheus_exporter(opentelemetry_sdk::metrics::Temporality::Cumulative, false);
        let provider = SdkMeterProvider::builder()
            .with_reader(exporter.clone())
            .with_resource(
                Resource::builder()
                    .with_attributes(vec![KeyValue::new("service.name", "test_service")])
                    .build(),
            )
            .build();
        opentelemetry::global::set_meter_provider(provider.clone());

        let meter = provider.meter("test_meter");
        let sum_metric = meter.i64_up_down_counter("test_non_monotonic_sum").build();
        sum_metric.add(10, &[KeyValue::new("key", "value")]);
        sum_metric.add(-5, &[KeyValue::new("key", "value")]);

        let encoded = collect_and_encode(&registry);

        assert!(encoded.contains(r#"test_non_monotonic_sum{otel_scope_name="test_meter",key="value"} 5"#));
        assert!(
            !encoded.contains("test_non_monotonic_sum_total"),
            "Non-monotonic sum should not have '_total' suffix"
        );
    }

    #[test]
    fn test_escape_key() {
        assert_eq!(escape_key("valid_key"), "valid_key");
        assert_eq!(escape_key("123start"), "_123start");
        assert_eq!(escape_key("key with spaces"), "key_with_spaces");
        assert_eq!(escape_key("key_with:dots"), "key_with:dots");
        assert_eq!(escape_key("!@#$%"), "_____");
    }
}
