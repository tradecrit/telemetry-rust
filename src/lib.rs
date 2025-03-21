pub mod profiler;
pub mod metrics;
mod logs;
mod tracer;

use std::env;

use crate::logs::init_log_provider;
use crate::metrics::{init_meter_provider, Metrics};
use crate::tracer::init_tracer_provider;

use opentelemetry::trace::TracerProvider;
use opentelemetry::KeyValue;
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::Protocol;
use opentelemetry_sdk::logs::SdkLoggerProvider;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_sdk::trace::{SdkTracerProvider, Tracer};
use opentelemetry_sdk::Resource;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

#[derive(Debug, Clone)]
pub struct TelemetryProvider {
    pub meter_provider: SdkMeterProvider,
    pub logger_provider: SdkLoggerProvider,
    pub tracer_provider: SdkTracerProvider,
    pub metrics: Metrics,
}

#[derive(Debug, Clone)]
pub struct TelemetryProviderConfig {
    pub trace_url: String,
    pub log_url: String,
    pub metric_url: String,
    pub protocol: Protocol
}

impl TelemetryProvider {
    pub fn new(config: TelemetryProviderConfig, attributes: Vec<KeyValue>) -> Self {
        let resource = Resource::builder()
            .with_attributes(attributes.clone())
            .build();

        let logger_provider = init_log_provider(config.log_url, resource.clone(), config.protocol);
        let logger_layer = OpenTelemetryTracingBridge::new(&logger_provider);

        let tracer_provider: SdkTracerProvider = init_tracer_provider(config.trace_url, resource.clone(), config.protocol);
        let tracer: Tracer = tracer_provider.tracer("app");
        let tracer_layer = OpenTelemetryLayer::new(tracer);

        let env_filter = EnvFilter::try_from_default_env()
            .or_else(|_| EnvFilter::try_new("info"))
            .expect("Failed to create EnvFilter");

        let meter_provider: SdkMeterProvider = init_meter_provider(config.metric_url, resource.clone(), config.protocol);

        let fmt_layer = tracing_subscriber::fmt::layer()
            .with_target(true)
            .with_thread_ids(true)
            .with_thread_names(true)
            .with_timer(tracing_subscriber::fmt::time::ChronoUtc::rfc_3339())
            .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
            .with_level(true)
            .with_ansi(true);

        tracing_subscriber::registry()
            .with(logger_layer)
            .with(tracer_layer)
            .with(fmt_layer)
            .with(env_filter)
            .init();

        let metrics: Metrics = Metrics::new(attributes.clone());

        Self {
            meter_provider,
            logger_provider,
            tracer_provider,
            metrics
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


impl Default for TelemetryProvider {
    fn default() -> Self {
        let app_name = env::var("APP_NAME").unwrap_or("application".to_string());
        let environment = env::var("ENVIRONMENT").unwrap_or("development".to_string());
        let app_version = env!("CARGO_PKG_VERSION");
        let job = env!("CARGO_PKG_NAME");

        let default_attributes = vec![
            KeyValue::new("service.name", app_name.clone()),
            KeyValue::new("service.version", app_version),
            KeyValue::new("service.environment", environment),
            KeyValue::new("job", job)
        ];

        let telemetry_protocol = env::var("TELEMETRY_PROTOCOL").unwrap_or("grpc".to_string());

        let telemetry_config = match telemetry_protocol.as_str() {
            "grpc" => {
                TelemetryProviderConfig {
                    trace_url: env::var("TRACE_URL").unwrap_or("http://127.0.0.1:4317".to_string()),
                    log_url: env::var("LOG_URL").unwrap_or("http://127.0.0.1:4317".to_string()),
                    metric_url: env::var("METRICS_URL").unwrap_or("http://127.0.0.1:4317".to_string()),
                    protocol: Protocol::Grpc
                }
            },
            "http" => {
                TelemetryProviderConfig {
                    trace_url: env::var("TRACE_URL").unwrap_or("http://localhost:4318/v1/traces".to_string()),
                    log_url: env::var("LOG_URL").unwrap_or("http://localhost:4318/v1/logs".to_string()),
                    metric_url: env::var("METRICS_URL").unwrap_or("http://localhost:4318/v1/metrics".to_string()),
                    protocol: Protocol::HttpJson
                }
            },
            _ => {
                panic!("Invalid telemetry protocol");
            }
        };

        let telemetry_provider: TelemetryProvider = TelemetryProvider::new(telemetry_config, default_attributes);

        telemetry_provider
    }
}
