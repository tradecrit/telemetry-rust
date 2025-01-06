use opentelemetry::global;
use opentelemetry::metrics::{Counter, Histogram, MeterProvider};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::metrics::{MeterProviderBuilder, PeriodicReader, SdkMeterProvider};
use opentelemetry_sdk::{runtime, Resource};

#[derive(Clone, Debug)]
pub struct Metrics {
    pub request_counter: Counter<u64>,
    pub request_duration: Histogram<f64>,
    pub request_ok_counter: Counter<u64>,
    pub request_error_counter: Counter<u64>,
}

pub(crate) fn init_meter_provider(collector_url: String, resource: Resource) -> SdkMeterProvider {
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


impl Metrics {
    pub fn new(app_name: &'static str, meter_provider: SdkMeterProvider) -> Self {
        let meter = meter_provider.meter(app_name);

        let request_counter = meter
            .u64_counter("requests_total")
            .with_description("Total number of requests")
            .build();

        let request_duration = meter
            .f64_histogram("request_duration_seconds")
            .with_description("Request duration in seconds")
            .build();

        let request_ok_counter = meter
            .u64_counter("requests_ok_total")
            .with_description("Total number of successful requests")
            .build();

        let request_error_counter = meter
            .u64_counter("requests_error_total")
            .with_description("Total number of failed requests")
            .build();

        Metrics {
            request_counter,
            request_duration,
            request_ok_counter,
            request_error_counter,
        }
    }
}


// pub struct MyService {
//     metrics: Metrics,
// }
//
// #[tonic::async_trait]
// impl MyService {
//     pub async fn new(metrics: Metrics) -> Self {
//         MyService { metrics }
//     }
//
//     pub async fn handle_request(&self, request: Request<()>) -> Result<Response<()>, Status> {
//         // Increment the total request counter
//         self.metrics.request_counter.add(1, &[]);
//
//         // Simulate request processing
//         let result = process_request(request).await;
//
//         match result {
//             Ok(_) => {
//                 // Increment the successful request counter
//                 self.metrics.request_ok_counter.add(1, &[]);
//                 Ok(Response::new(()))
//             }
//             Err(_) => {
//                 // Increment the failed request counter
//                 self.metrics.request_error_counter.add(1, &[]);
//                 Err(Status::internal("Request failed"))
//             }
//         }
//     }
// }
//
// async fn process_request(request: Request<()>) -> Result<(), ()> {
//     // Simulate some processing logic
//     Ok(())
// }
