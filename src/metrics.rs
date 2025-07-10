use opentelemetry::global;
use opentelemetry_otlp::{MetricExporter, WithExportConfig};
use opentelemetry_sdk::metrics::{MeterProviderBuilder, PeriodicReader, SdkMeterProvider};
use opentelemetry_sdk::Resource;

pub(crate) fn init_meter_provider(collector_url: String, resource: Resource) -> SdkMeterProvider {
    let exporter: MetricExporter = opentelemetry_otlp::MetricExporter::builder()
        .with_tonic()
        .with_endpoint(collector_url)
        .with_protocol(opentelemetry_otlp::Protocol::Grpc)
        .with_timeout(std::time::Duration::from_secs(5))
        .build()
        .expect("Failed to create OTLP metric exporter");

    let reader = PeriodicReader::builder(exporter)
        .with_interval(std::time::Duration::from_secs(1))
        .build();

    // let stdout_reader =
    //     PeriodicReader::builder(opentelemetry_stdout::MetricExporter::default()).build();

    let provider: SdkMeterProvider = MeterProviderBuilder::default()
        .with_resource(resource)
        .with_reader(reader)
        .build();

    global::set_meter_provider(provider.clone());

    provider
}
