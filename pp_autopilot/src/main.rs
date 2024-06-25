use tracing::{event, Level};

#[tokio::main]
async fn main() {
    event!(Level::INFO, "Planepilot started",);

    dotenv::dotenv().ok();

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info")
    }

    pp_autopilot::utils::start_tracing_subscriber();

    match pp_autopilot::run_app().await {
        Ok(_) => {}
        Err(e) => {
            println!("Error: {}", e)
        }
    }
}
