
use pp_dataserver::utils;
use dotenv::dotenv;

#[tokio::main]
async fn main() {
    dotenv().ok();
    utils::trace::start_tracing_subscriber();
    
    match pp_dataserver::run_app().await {
        Ok(()) => {}
        Err(e) => panic!("Error in main program: {}", e),
    }
}

