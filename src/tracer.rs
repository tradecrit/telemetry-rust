use std::time::Duration;
use opentelemetry::global;
use opentelemetry_otlp::{WithExportConfig};
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::{trace, Resource};
use opentelemetry_sdk::trace::SdkTracerProvider;

pub fn init_tracer_provider(collector_url: String, resource: Resource) -> SdkTracerProvider {
    global::set_text_map_propagator(TraceContextPropagator::new());

    let exporter: opentelemetry_otlp::SpanExporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(collector_url)
        .with_protocol(opentelemetry_otlp::Protocol::Grpc)
        .with_timeout(Duration::from_secs(3))
        .build()
        .expect("Failed to create OTLP span exporter");

    let provider = SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_id_generator(trace::RandomIdGenerator::default())
        .with_max_attributes_per_span(16)
        .with_max_events_per_span(16)
        .with_max_links_per_span(16)
        .with_resource(resource)
        .build();

    global::set_tracer_provider(provider.clone());

    provider
}
