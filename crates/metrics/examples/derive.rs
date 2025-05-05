use example::Kind;
use opentelemetry::KeyValue;
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::metrics::{PeriodicReader, SdkMeterProvider};

#[scuffle_metrics::metrics]
mod example {
    use scuffle_metrics::{CounterU64, MetricEnum};

    #[derive(MetricEnum)]
    pub enum Kind {
        Http,
        Grpc,
    }

    /// Requests for adding 2 numbers
    #[metrics(unit = "requests")]
    pub fn add(a: u64, b: u64, kind: Kind) -> CounterU64;
}

#[tokio::main]
async fn main() {
    let exporter = opentelemetry_stdout::MetricExporterBuilder::default().build();

    let provider = SdkMeterProvider::builder()
        .with_resource(
            Resource::builder()
                .with_attribute(KeyValue::new("service.name", env!("CARGO_BIN_NAME")))
                .build(),
        )
        .with_reader(PeriodicReader::builder(exporter).build())
        .build();

    opentelemetry::global::set_meter_provider(provider.clone());

    example::add(1, 2, Kind::Http).incr();

    for i in 0..10 {
        example::add(i, i, Kind::Http).incr();
    }

    provider.shutdown().unwrap();
}
