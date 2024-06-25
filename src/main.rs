#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "trace");
    }

    start_tracing_subscriber();

    tokio::select! {
        _ = pp_planeconnector::run_app() => {
        },
        _ = pp_dataserver::run_app() => {
        },
        _ = pp_autopilot::run_app() => {
        }
    }
}

fn start_tracing_subscriber() {
    tracing_subscriber::fmt::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
}