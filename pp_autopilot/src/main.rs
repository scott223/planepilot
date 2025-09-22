use tracing::{event, Level};

#[tokio::main]
async fn main() {
    event!(Level::INFO, "Planepilot started",);

    dotenv::dotenv().ok();

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info")
    }

    pp_autopilot::utils::start_tracing_subscriber();

    //data server, planeconnector, autopilot
    let service_adresses: (String, String, String) = (
        "http://localhost:3000/api/v1".to_owned(),
        "http://localhost:3100/api/v1".to_owned(),
        "http://localhost:3200/api/v1".to_owned(),
    );

    match pp_autopilot::run_app(&service_adresses).await {
        Ok(_) => {}
        Err(e) => {
            println!("Error: {}", e)
        }
    }
}
