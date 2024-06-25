use tracing::{event, Level};

#[tokio::main]
async fn main() {
    event!(Level::INFO, "Planepilot started",);

    dotenv::dotenv().ok();

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info")
    }

    pp-autopilot::utils::start_tracing_subscriber();

    match pp-autopilot::run_app().await {
        Ok(_) => {}
        Err(e) => {
            println!("Error: {}", e)
        }
    }
}
