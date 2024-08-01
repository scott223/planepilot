
use pp_dataserver::utils;
use dotenv::dotenv;

#[tokio::main]
async fn main() {
    dotenv().ok();
    utils::trace::start_tracing_subscriber();
    
    //data server, planeconnector, autopilot
    let service_adresses: (String, String, String) = (
        "http://localhost:3000/api/v1".to_owned(),
        "http://localhost:3100/api/v1".to_owned(),
        "http://localhost:3200/api/v1".to_owned(),
    );

    match pp_dataserver::run_app(&service_adresses).await {
        Ok(()) => {}
        Err(e) => panic!("Error in main program: {}", e),
    }
}

