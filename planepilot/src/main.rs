#[tokio::main]
async fn main() {
    println!("Planepilot started");

    dotenv::dotenv().ok();
    planepilot::utils::start_tracing_subscriber();

    match planepilot::run_app().await {
        Ok(_) => {}
        Err(e) => {
            println!("Error: {}", e)
        }
    }
}
