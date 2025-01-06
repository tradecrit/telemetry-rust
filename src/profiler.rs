use pyroscope::pyroscope::{PyroscopeAgentReady, PyroscopeAgentRunning};
use pyroscope::PyroscopeAgent;
use pyroscope_pprofrs::{pprof_backend, PprofConfig};

pub(crate) fn init_profiler(url: &str, service_name: &str) -> Result<PyroscopeAgent<PyroscopeAgentReady>, Box<dyn std::error::Error>> {
    let pprof_config = PprofConfig::new()
        .sample_rate(100)
        .report_thread_id()
        .report_thread_name();

    let backend_impl = pprof_backend(pprof_config);

    let tags: Vec<(&str, &str)> = vec![("service_name", service_name), ("language", "rust")];

    // Configure Pyroscope Agent
    let agent = PyroscopeAgent::builder(url, service_name)
        .backend(backend_impl)
        .tags(tags)
        .build()
        .expect("Failed to create Pyroscope Agent");

    Ok(agent)
}


pub async fn create(url: &str, service_name: &str) -> Result<PyroscopeAgent<PyroscopeAgentRunning>, Box<dyn std::error::Error>> {
    let agent_ready = init_profiler(
        url,
        service_name,
    ).expect("Failed to initialize profiler");

    let agent_running_guard = agent_ready.start()?;

    Ok(agent_running_guard)
}


pub async fn destroy(agent_running_guard: PyroscopeAgent<PyroscopeAgentRunning>) -> Result<(), Box<dyn std::error::Error>> {
    let agent_ready: PyroscopeAgent<PyroscopeAgentReady> = agent_running_guard.stop()?;
    agent_ready.shutdown();

    Ok(())
}
