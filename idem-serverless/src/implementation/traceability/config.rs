use serde::{Deserialize};

#[derive(Deserialize)]
pub struct TraceabilityHandlerConfig {
    pub enabled: bool,
    pub autogen_correlation_id: bool,
    pub correlation_header_name: String,
    pub traceability_header_name: String,
    pub add_trace_to_response: bool,
}

impl Default for TraceabilityHandlerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            autogen_correlation_id: true,
            traceability_header_name: "x-trace".into(),
            correlation_header_name: "x-correlation".into(),
            add_trace_to_response: true
        }
    }
}
