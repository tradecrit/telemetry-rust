# Opentelemetry Rust

## Overview

I was frustrated and wasted many days getting opentelemetry working in Rust as there are dozens of crates that
work in random combinations, random versions, and random configurations that are not documents and typically outdated.

After weeks I have a working baseline config that anyone is free to use, a single import of this crate will configure the app completely with an otel collected running on:

- `grpc://localhost:4317` for the telemetry (Trace, Metrics, Logs)

This is a batteries included, out of the box implementation of OpenTelemetry for Rust. It is designed to be a one stop shop for any OpenTelemetry user, providing a powerful, but simple API to interact with OpenTelemetry.

This gives out of the box support for the following:

- Tracing
- Metrics
- Logs

It also correlates it for the Rust app. This means that all the telemetry data is correlated and can be used to debug and monitor the Rust app.

## Usage

Simply import and keep the `TelemetryProvider` alive for the lifetime of the application. This will automatically start the OpenTelemetry SDK and start collecting telemetry data.

```
use telemetry_rust::TelemetryProvider;

use opentelemetry_semantic_conventions::{
    attribute::{SERVICE_NAME, SERVICE_VERSION, DEPLOYMENT_ENVIRONMENT_NAME},
    SCHEMA_URL,
};

...

let resource: Resource = Resource::from_schema_url(
        [
            KeyValue::new(SERVICE_NAME, "my-app"),
            KeyValue::new(SERVICE_VERSION, env!("CARGO_PKG_VERSION")),
            KeyValue::new(DEPLOYMENT_ENVIRONMENT_NAME, "development"),
        ],
        SCHEMA_URL,
    );

let telemetry_provider = TelemetryProvider::new("grpc://localhost:4317".to_string(), resource);
```

when you are done with the app, during the graceful shutdown, call the `shutdown` method on the `TelemetryProvider` to stop the OpenTelemetry SDK. This is not a well documented need by the opentelemetry crates but it flushes the ending data to the collector.

```
telemetry_provider.shutdown();
```
