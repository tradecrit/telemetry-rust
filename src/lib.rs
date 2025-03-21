pub mod profiler;
mod metrics;
mod logs;
mod tracer;

use std::env;
use crate::logs::init_log_provider;
use crate::metrics::init_meter_provider;
use crate::tracer::init_tracer_provider;
use opentelemetry::trace::TracerProvider;
use opentelemetry::{Key, KeyValue, Value};
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
pub struct TelemetryProviderAttributes {
    pub app_name: Value,
    pub app_version: Value,
    pub environment: Value,
}

#[derive(Debug, Clone)]
pub struct TelemetryProvider {
    pub meter_provider: SdkMeterProvider,
    pub logger_provider: SdkLoggerProvider,
    pub tracer_provider: SdkTracerProvider,
    pub attributes: TelemetryProviderAttributes
}

#[derive(Debug, Clone)]
pub struct TelemetryProviderConfig {
    pub resource: Resource,
    pub trace_url: String,
    pub log_url: String,
    pub metric_url: String,
    pub protocol: Protocol
}

impl TelemetryProvider {
    pub fn new(config: TelemetryProviderConfig) -> Self {
        let resource = config.resource;
        let parsed_app_name = resource.get(&Key::new("service.name")).expect("Failed to get service name");
        let parsed_app_version = resource.get(&Key::new("service.version")).expect("Failed to get service version");
        let parsed_environment = resource.get(&Key::new("service.environment")).expect("Failed to get service environment");

        let attributes = TelemetryProviderAttributes {
            app_name: parsed_app_name,
            app_version: parsed_app_version,
            environment: parsed_environment
        };

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

        Self {
            meter_provider,
            logger_provider,
            tracer_provider,
            attributes,
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

        let app_details_resource: Resource = Resource::builder()
            .with_attribute(KeyValue::new("service.name", app_name.clone()))
            .with_attribute(KeyValue::new("service.version", app_version))
            .with_attribute(KeyValue::new("service.environment", environment.clone()))
            .build();

        let telemetry_protocol = env::var("TELEMETRY_PROTOCOL").unwrap_or("grpc".to_string());

        let telemetry_config = match telemetry_protocol.as_str() {
            "grpc" => {
                TelemetryProviderConfig {
                    resource: app_details_resource,
                    trace_url: env::var("TRACE_URL").unwrap_or("http://127.0.0.1:4317".to_string()),
                    log_url: env::var("LOG_URL").unwrap_or("http://127.0.0.1:4317".to_string()),
                    metric_url: env::var("METRICS_URL").unwrap_or("http://127.0.0.1:4317".to_string()),
                    protocol: Protocol::Grpc
                }
            },
            "http" => {
                TelemetryProviderConfig {
                    resource: app_details_resource,
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

        let telemetry_provider: TelemetryProvider = TelemetryProvider::new(telemetry_config);

        telemetry_provider
    }
}
