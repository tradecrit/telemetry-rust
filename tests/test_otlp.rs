use std::time::Duration;
use opentelemetry::{global, KeyValue};
use telemetry_rust::{TelemetryProvider, TelemetryProviderConfig};

fn init_provider() -> TelemetryProvider {
    dotenvy::dotenv().ok();

    let telemetry_url = std::env::var("TELEMETRY_URL")
        .expect("TELEMETRY_URL must be set")
        .trim_matches(|c| c == '"' || c == '\'')
        .to_string();

    let default_attributes = vec![
        KeyValue::new("service.name", "otel"),
    ];

    let telemetry_config = TelemetryProviderConfig {
        trace_url: telemetry_url.clone(),
        log_url: telemetry_url.clone(),
        metric_url: telemetry_url.clone(),
        protocol: opentelemetry_otlp::Protocol::Grpc,
    };

    let telemetry_provider: TelemetryProvider = TelemetryProvider::new(telemetry_config, default_attributes);

    telemetry_provider
}

#[tokio::test]
async fn test_telemetry_provider_init() {
    let telemetry_provider = init_provider();

    tracing::info!("Hello, world!");

    let try_shutdown = telemetry_provider.shutdown();

    assert!(try_shutdown.is_ok());
}


#[tokio::test]
async fn test_metrics_visible() {
    let telemetry_provider = init_provider();

    let meter = global::meter("test");
    let counter = meter
        .u64_counter("test_requests_total")
        .with_description("Test requests")
        .build();

    for i in 0..10 {
        counter.add((10 * i as u64) as u64, &[KeyValue::new("test", "true")]);
    }

    tokio::time::sleep(Duration::from_secs(10)).await;

    telemetry_provider.shutdown().unwrap();
}
