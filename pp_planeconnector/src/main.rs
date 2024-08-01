#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    pp_planeconnector::utils::start_tracing_subscriber();

    match pp_planeconnector::run_app().await {
        Ok(()) => {}
        Err(e) => panic!("Error in main program: {}", e),
    }
}