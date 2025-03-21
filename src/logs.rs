use std::time::Duration;
use opentelemetry_otlp::{LogExporter, WithExportConfig};
use opentelemetry_sdk::logs::{BatchConfigBuilder, BatchLogProcessor, LoggerProviderBuilder, SdkLoggerProvider};
use opentelemetry_sdk::Resource;


pub fn init_log_provider(log_url: String, resource: Resource) -> SdkLoggerProvider {
    let exporter: LogExporter = opentelemetry_otlp::LogExporter::builder()
        .with_tonic()
        .with_endpoint(log_url)
        .with_protocol(opentelemetry_otlp::Protocol::Grpc)
        .with_timeout(Duration::from_secs(5))
        .build()
        .expect("Failed to create OTLP log exporter");

    let processor = BatchLogProcessor::builder(exporter)
        .with_batch_config(
            BatchConfigBuilder::default()
                .with_max_queue_size(2048)
                .with_max_export_batch_size(512)
                .with_scheduled_delay(Duration::from_secs(1))
                .build(),
        )
        .build();


    let provider: SdkLoggerProvider = LoggerProviderBuilder::default()
        .with_log_processor(processor)
        .with_resource(resource)
        .build();

    provider
}
