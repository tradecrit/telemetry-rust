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

            telemetry_provider.metrics.increment_request_counter(random_1_to_10);

            test_instrument(&format!("Request count: {}", random_1_to_10));

            count += 1;

            if count > 10 {
                break;
            }
        }

        telemetry_provider.shutdown();
    }
}
