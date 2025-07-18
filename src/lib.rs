mod logs;
pub mod metrics;
pub mod profiler;
mod tracer;

use std::env;

use crate::logs::init_log_provider;
use crate::metrics::init_meter_provider;
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
use tracing_subscriber::{EnvFilter};

#[derive(Debug, Clone)]
pub struct TelemetryProvider {
    pub meter_provider: SdkMeterProvider,
    pub logger_provider: SdkLoggerProvider,
    pub tracer_provider: SdkTracerProvider,
}

#[derive(Debug, Clone)]
pub struct TelemetryProviderConfig {
    pub trace_url: String,
    pub log_url: String,
    pub metric_url: String,
    pub protocol: Protocol,
}

impl TelemetryProvider {
    pub fn new(config: TelemetryProviderConfig, attributes: Vec<KeyValue>) -> Self {
        let resource = Resource::builder()
            .with_attributes(attributes.clone())
            .build();

        let env_filter = EnvFilter::try_from_default_env()
            .or_else(|_| EnvFilter::try_new("info"))
            .expect("Failed to create EnvFilter");

        let logger_provider: SdkLoggerProvider = init_log_provider(config.log_url, resource.clone());

        let logger_layer = OpenTelemetryTracingBridge::new(&logger_provider);

        let tracer_provider: SdkTracerProvider = init_tracer_provider(config.trace_url, resource.clone());
        let tracer: Tracer = tracer_provider.tracer("app");
        let tracer_layer = OpenTelemetryLayer::new(tracer);

        let meter_provider: SdkMeterProvider =
            init_meter_provider(config.metric_url, resource.clone());

        let fmt_layer = tracing_subscriber::fmt::layer()
            .with_target(true)
            .with_thread_ids(true)
            .with_thread_names(true)
            .with_line_number(true)
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
        }
    }

    pub fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Err(err) = self.tracer_provider.shutdown() {
            tracing::warn!("Failed to shutdown tracer provider: {}", err);
        }
        if let Err(err) = self.meter_provider.shutdown() {
            tracing::warn!("Failed to shutdown meter provider: {}", err);
        }
        // Do not attempt to shut down the logger, the tracing bridge does some
        // broken stuff behind the scenes on it.

        Ok(())
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
            KeyValue::new("job", job),
        ];

        let telemetry_url =
            env::var("TELEMETRY_URL").unwrap_or("http://localhost:4317".to_string());
        let telemetry_protocol = env::var("TELEMETRY_PROTOCOL").unwrap_or("grpc".to_string());

        let telemetry_config = match telemetry_protocol.as_str() {
            "grpc" => TelemetryProviderConfig {
                trace_url: telemetry_url.clone(),
                log_url: telemetry_url.clone(),
                metric_url: telemetry_url.clone(),
                protocol: Protocol::Grpc,
            },
            "http" => TelemetryProviderConfig {
                trace_url: telemetry_url.clone(),
                log_url: telemetry_url.clone(),
                metric_url: telemetry_url.clone(),
                protocol: Protocol::HttpJson,
            },
            _ => {
                panic!("Invalid telemetry protocol");
            }
        };

        let telemetry_provider: TelemetryProvider =
            TelemetryProvider::new(telemetry_config, default_attributes);

        telemetry_provider
    }
}
