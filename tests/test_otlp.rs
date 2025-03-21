#[cfg(test)]
mod tests {
    use dotenvy::dotenv;

    use telemetry_rust::TelemetryProvider;

    #[tracing::instrument]
    fn test_instrument(msg: &str) {
        tracing::info!("Tracing: {}", msg);
    }

    #[tokio::test]
    async fn test_tonic_metrics() {
        dotenv().ok();

        let telemetry_provider: TelemetryProvider = TelemetryProvider::default();

        let mut count = 0;

        loop {
            let random_1_to_10 = rand::random::<u64>() % 10 + 1;

            telemetry_provider.metrics.increment_request_counter();

            test_instrument(&format!("Request count: {}", random_1_to_10));

            count += 1;

            if count > 100 {
                break;
            }

            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }

        telemetry_provider.shutdown();
    }
}
