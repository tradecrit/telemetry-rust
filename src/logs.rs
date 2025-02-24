use std::time::Duration;
use opentelemetry_otlp::{LogExporter, WithExportConfig};
use opentelemetry_sdk::logs::{LoggerProviderBuilder, SdkLoggerProvider};
use opentelemetry_sdk::Resource;


pub fn init_log_provider(log_url: String, resource: Resource) -> SdkLoggerProvider {
    let exporter: LogExporter = opentelemetry_otlp::LogExporter::builder()
        .with_tonic()
        .with_endpoint(log_url)
        .with_protocol(opentelemetry_otlp::Protocol::Grpc)
        .with_timeout(Duration::from_secs(3))
        .build()
        .expect("Failed to create OTLP log exporter");

    let provider: SdkLoggerProvider = LoggerProviderBuilder::default()
        .with_batch_exporter(exporter)
        .with_resource(resource)
        .build();

    provider
}
