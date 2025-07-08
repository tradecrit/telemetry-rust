use opentelemetry_otlp::{MetricExporter, Protocol, WithExportConfig};
use opentelemetry_sdk::metrics::{MeterProviderBuilder, PeriodicReader, SdkMeterProvider};
use opentelemetry_sdk::Resource;

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

    let reader = PeriodicReader::builder(exporter)
        .with_interval(std::time::Duration::from_secs(5))
        .build();

    let stdout_reader =
        PeriodicReader::builder(opentelemetry_stdout::MetricExporter::default()).build();

    let provider = MeterProviderBuilder::default()
        .with_resource(resource)
        .with_reader(reader)
        .with_reader(stdout_reader)
        .build();

    provider
}
