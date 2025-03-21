use opentelemetry::metrics::Counter;
use opentelemetry::{global, KeyValue};
use opentelemetry_otlp::{MetricExporter, Protocol, WithExportConfig};
use opentelemetry_sdk::metrics::{MeterProviderBuilder, PeriodicReader, SdkMeterProvider};
use opentelemetry_sdk::Resource;

#[derive(Debug, Clone)]
pub struct Metrics {
    attributes: Vec<KeyValue>,
    request_counter: Counter<u64>,
}

impl Metrics {
    pub fn new(attributes: Vec<KeyValue>) -> Self {
        let meter = global::meter("global");

        let request_counter = meter.u64_counter("request.count").build();

        Metrics {
            attributes,
            request_counter,
        }
    }


    pub fn increment_request_counter(&self) {
        // Increment the counter each time a request is made
        self.request_counter.add(1, &self.attributes);
    }
}

pub(crate) fn init_meter_provider(collector_url: String, resource: Resource, protocol: Protocol) -> SdkMeterProvider {
    let exporter: MetricExporter = match protocol {
        Protocol::Grpc => {
            opentelemetry_otlp::MetricExporter::builder()
                .with_tonic()
                .with_endpoint(collector_url)
                .with_protocol(opentelemetry_otlp::Protocol::Grpc)
                .with_timeout(std::time::Duration::from_secs(5))
                .build()
                .expect("Failed to create OTLP metric exporter")
        },
        Protocol::HttpJson => {
            opentelemetry_otlp::MetricExporter::builder()
                .with_http()
                .with_endpoint(collector_url)
                .with_protocol(opentelemetry_otlp::Protocol::HttpJson)
                .with_temporality(opentelemetry_sdk::metrics::Temporality::default())
                .build()
                .expect("Failed to create OTLP metric exporter")
        },
        _ => panic!("Unsupported protocol"),
    };

    let reader: PeriodicReader = PeriodicReader::builder(exporter)
        .with_interval(std::time::Duration::from_secs(5))
        .build();

    let stdout_reader: PeriodicReader =
        PeriodicReader::builder(opentelemetry_stdout::MetricExporter::default()).build();

    let meter_provider = MeterProviderBuilder::default()
        .with_resource(resource)
        .with_reader(reader)
        .with_reader(stdout_reader)
        .build();

    global::set_meter_provider(meter_provider.clone());

    meter_provider
}
