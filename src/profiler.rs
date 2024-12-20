use pyroscope::pyroscope::PyroscopeAgentReady;
use pyroscope::PyroscopeAgent;
use pyroscope_pprofrs::{pprof_backend, PprofConfig};

pub fn create(url: &str, service_name: &str) -> Result<PyroscopeAgent<PyroscopeAgentReady>, Box<dyn std::error::Error>> {
    let pprof_config = PprofConfig::new()
        .sample_rate(100)
        .report_thread_id()
        .report_thread_name();

    let backend_impl = pprof_backend(pprof_config);

    let tags: Vec<(&str, &str)> = vec![("service.name", service_name), ("language", "rust")];

    // Configure Pyroscope Agent
    let agent = PyroscopeAgent::builder(url, service_name)
        .backend(backend_impl)
        .tags(tags)
        .build()
        .expect("Failed to create Pyroscope Agent");

    Ok(agent)
}
