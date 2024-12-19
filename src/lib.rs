pub mod profiler;

use opentelemetry::global;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::logs::LoggerProvider;
use opentelemetry_sdk::metrics::{MeterProviderBuilder, PeriodicReader, SdkMeterProvider};
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::trace::{Tracer, TracerProvider};
use opentelemetry_sdk::{runtime, trace, Resource};
use std::time::Duration;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

#[derive(Debug, Clone)]
pub struct TelemetryProvider {
    pub meter_provider: SdkMeterProvider,
    pub logger_provider: LoggerProvider,
    pub tracer_provider: TracerProvider,
}

#[derive(Debug, Clone)]
pub struct TelemetryProviderConfig {
    pub resource: Resource,
    pub trace_url: String,
    pub log_url: String,
    pub metric_url: String,
}

impl TelemetryProvider {
    pub fn new(config: TelemetryProviderConfig) -> Self {
        let resource = config.resource;

        let logger_provider: LoggerProvider = init_logs(config.log_url, resource.clone());
        let logger_layer = OpenTelemetryTracingBridge::new(&logger_provider);

        let fmt_layer = tracing_subscriber::fmt::layer()
            .with_target(true)
            .with_thread_ids(true)
            .with_thread_names(true)
            .with_timer(tracing_subscriber::fmt::time::ChronoUtc::rfc_3339())
            .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
            .with_level(true)
            .with_ansi(true);

        let tracer_provider: TracerProvider = init_tracer(config.trace_url, resource.clone());
        let tracer: Tracer = tracer_provider.tracer("app");
        let tracer_layer = OpenTelemetryLayer::new(tracer);

        let env_filter = EnvFilter::try_from_default_env()
            .or_else(|_| EnvFilter::try_new("info"))
            .expect("Failed to create EnvFilter");

        tracing_subscriber::registry()
            .with(logger_layer)
            .with(tracer_layer)
            .with(fmt_layer)
            .with(env_filter)
            .init();

        let meter_provider: SdkMeterProvider =
            init_meter_provider(config.metric_url, resource.clone());

        Self {
            meter_provider,
            logger_provider,
            tracer_provider,
        }
    }

    pub fn shutdown(&self) {
        self.tracer_provider
            .shutdown()
            .expect("TracerProvider should shutdown successfully");
        self.meter_provider
            .shutdown()
            .expect("MeterProvider should shutdown successfully");
        self.logger_provider
            .shutdown()
            .expect("LoggerProvider should shutdown successfully");
    }
}

fn init_tracer(collector_url: String, resource: Resource) -> TracerProvider {
    global::set_text_map_propagator(TraceContextPropagator::new());

    let exporter: opentelemetry_otlp::SpanExporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(collector_url)
        .with_protocol(opentelemetry_otlp::Protocol::Grpc)
        .with_timeout(Duration::from_secs(3))
        .build()
        .expect("Failed to create OTLP span exporter");

    let provider = TracerProvider::builder()
        .with_batch_exporter(exporter, opentelemetry_sdk::runtime::Tokio)
        .with_id_generator(trace::RandomIdGenerator::default())
        .with_max_attributes_per_span(16)
        .with_max_events_per_span(16)
        .with_max_links_per_span(16)
        .with_resource(resource)
        .build();

    global::set_tracer_provider(provider.clone());

    provider
}

// Construct MeterProvider for MetricsLayer
fn init_meter_provider(collector_url: String, resource: Resource) -> SdkMeterProvider {
    let exporter = opentelemetry_otlp::MetricExporter::builder()
        .with_tonic()
        .with_endpoint(collector_url)
        .with_protocol(opentelemetry_otlp::Protocol::Grpc)
        .with_temporality(opentelemetry_sdk::metrics::Temporality::default())
        .build()
        .expect("Failed to create OTLP metric exporter");

    let reader = PeriodicReader::builder(exporter, runtime::Tokio)
        .with_interval(std::time::Duration::from_secs(30))
        .build();

    // For debugging in development
    let stdout_reader = PeriodicReader::builder(
        opentelemetry_stdout::MetricExporter::default(),
        runtime::Tokio,
    )
    .build();

    let meter_provider = MeterProviderBuilder::default()
        .with_resource(resource)
        .with_reader(reader)
        .with_reader(stdout_reader)
        .build();

    global::set_meter_provider(meter_provider.clone());

    meter_provider
}

fn init_logs(collector_url: String, resource: Resource) -> LoggerProvider {
    let exporter: opentelemetry_otlp::LogExporter = opentelemetry_otlp::LogExporter::builder()
        .with_tonic()
        .with_endpoint(collector_url)
        .with_protocol(opentelemetry_otlp::Protocol::Grpc)
        .with_timeout(std::time::Duration::from_secs(3))
        .build()
        .expect("Failed to create OTLP log exporter");

    LoggerProvider::builder()
        .with_batch_exporter(exporter, opentelemetry_sdk::runtime::Tokio)
        .with_resource(resource)
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use opentelemetry::KeyValue;
    use opentelemetry_semantic_conventions::resource::SERVICE_NAME;
    use opentelemetry_semantic_conventions::SCHEMA_URL;

    #[tokio::test]
    async fn test_telemetry_provider() {
        let resource: Resource =
            Resource::from_schema_url([KeyValue::new(SERVICE_NAME, "test-service")], SCHEMA_URL);

        let telemetry_provider_config: TelemetryProviderConfig = TelemetryProviderConfig {
            resource: resource.clone(),
            trace_url: "grpc://localhost:4317".to_string(),
            log_url: "grpc://localhost:4300".to_string(),
            metric_url: "grpc://localhost:9009".to_string(),
        };

        let telemetry_provider = TelemetryProvider::new(telemetry_provider_config);

        tracing::info!("Test event");

        telemetry_provider.shutdown();
    }
}
