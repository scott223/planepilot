pub fn start_tracing_subscriber() {
    // initialize tracing
    tracing_subscriber::fmt::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
}
