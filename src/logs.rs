use std::time::Duration;
use opentelemetry_otlp::{LogExporter, WithExportConfig};
use opentelemetry_sdk::logs::{BatchConfig, BatchConfigBuilder, BatchLogProcessor, LoggerProviderBuilder, SdkLoggerProvider};
use opentelemetry_sdk::Resource;

pub(crate) fn init_log_provider(log_url: String, resource: Resource) -> SdkLoggerProvider {
    let exporter: LogExporter = opentelemetry_otlp::LogExporter::builder()
        .with_tonic()
        .with_endpoint(log_url)
        .with_protocol(opentelemetry_otlp::Protocol::Grpc)
        .with_timeout(Duration::from_secs(5))
        .build()
        .expect("Failed to create OTLP log exporter");

    let batch_config: BatchConfig = BatchConfigBuilder::default()
        .with_max_queue_size(4096)
        .with_max_export_batch_size(512)
        .with_scheduled_delay(Duration::from_millis(500))
        .build();

    let processor: BatchLogProcessor = BatchLogProcessor::builder(exporter)
        .with_batch_config(batch_config)
        .build();

    let provider: SdkLoggerProvider = LoggerProviderBuilder::default()
        .with_log_processor(processor)
        .with_resource(resource)
        .build();

    provider
}
