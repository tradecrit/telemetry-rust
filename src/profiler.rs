use pyroscope::pyroscope::PyroscopeAgentReady;
use pyroscope::PyroscopeAgent;
use pyroscope_pprofrs::{pprof_backend, PprofConfig};

pub fn create(url: &str, service_name: String) -> Result<PyroscopeAgent<PyroscopeAgentReady>, Box<dyn std::error::Error>> {
    let pprof_config = PprofConfig::new()
        .sample_rate(100)
        .report_thread_id()
        .report_thread_name();

    let backend_impl = pprof_backend(pprof_config);

    // Configure Pyroscope Agent
    let agent = PyroscopeAgent::builder(url, &service_name)
        .backend(backend_impl)
        .build()
        .expect("Failed to create Pyroscope Agent");

    Ok(agent)
}
