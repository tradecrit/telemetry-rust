#[cfg(test)]
mod tests {
    use tracing::{span, Level};
    use telemetry_rust::TelemetryProvider;

    #[tokio::test]
    async fn test_provider() {
        // let telemetry_provider: TelemetryProvider = TelemetryProvider::default();
        //
        // tracing::info!("Hello world!");
        //
        // let my_span = span!(Level::INFO, "my_span", answer = 42);
        //
        // telemetry_provider.shutdown();
    }
}
