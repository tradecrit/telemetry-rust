#[cfg(test)]
mod tests {
    use dotenvy::dotenv;
    use telemetry_rust::TelemetryProvider;
    
    #[tracing::instrument]
    fn test_instrument() {
        tracing::info!("hello world");
    }

    #[tokio::test]
    async fn test_provider() {
        dotenv().ok();

        let telemetry_provider: TelemetryProvider = TelemetryProvider::default();

        test_instrument();

        telemetry_provider.shutdown();
    }
}
