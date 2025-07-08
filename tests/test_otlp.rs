use std::time::Duration;
use opentelemetry::KeyValue;
use telemetry_rust::{TelemetryProvider, TelemetryProviderConfig};

#[tokio::test]
async fn test_otlp() {
    dotenvy::dotenv().ok();

    let telemetry_url = std::env::var("TELEMETRY_URL")
        .expect("TELEMETRY_URL must be set")
        .trim_matches(|c| c == '"' || c == '\'')
        .to_string();

    let default_attributes = vec![
        KeyValue::new("service.name", "market_maker"),
    ];

    let telemetry_config = TelemetryProviderConfig {
        trace_url: telemetry_url.clone(),
        log_url: telemetry_url.clone(),
        metric_url: telemetry_url.clone(),
        protocol: opentelemetry_otlp::Protocol::Grpc,
    };

    let telemetry_provider: TelemetryProvider = TelemetryProvider::new(telemetry_config, default_attributes);

    tokio::time::sleep(Duration::from_millis(100)).await;

    tracing::info!("Hello, world!");

    let try_shutdown = telemetry_provider.shutdown();

    assert!(try_shutdown.is_ok());
}
