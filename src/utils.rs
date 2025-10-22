use tower_http::{
    classify::{ServerErrorsAsFailures, SharedClassifier},
    trace::TraceLayer,
};
use tracing::Level;

// initiate tracing

pub fn start_tracing_subscriber() {
    tracing_subscriber::fmt::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
}

// prepare a trace layer for the http server that wlil connect the server to tracing

pub fn return_trace_layer() -> TraceLayer<SharedClassifier<ServerErrorsAsFailures>> {
    TraceLayer::new_for_http()
        .make_span_with(tower_http::trace::DefaultMakeSpan::new().include_headers(true))
        .on_request(tower_http::trace::DefaultOnRequest::new().level(Level::TRACE))
        .on_response(
            tower_http::trace::DefaultOnResponse::new()
                .level(Level::TRACE)
                .latency_unit(tower_http::LatencyUnit::Micros),
        ) //todo on error, etc
}
