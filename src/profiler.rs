// use opentelemetry_sdk::Resource;
// use pyroscope::pyroscope::PyroscopeAgentReady;
// use pyroscope::PyroscopeAgent;
// use pyroscope_pprofrs::{pprof_backend, PprofConfig};
//
// pub(crate) fn init_profiler(collector_url: String, app_name: String) -> PyroscopeAgent<PyroscopeAgentReady> {
//     let config = PprofConfig::new()
//         .sample_rate(1000)
//         .report_thread_id()
//         .report_thread_name();
//
//     let agent = PyroscopeAgent::builder(collector_url, app_name)
//             .backend(pprof_backend(config))
//             .build()
//             .expect("failed to create pyroscope agent");
//
//     agent
// }
