use std::collections::HashMap;

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde_json::{json, Value};

#[derive(Debug, Clone)]
pub struct AppState {
    pub plane_state: HashMap<String, Value>,
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let mut app_state = AppState {
        plane_state: HashMap::new(),
    };

    app_state.plane_state.insert(
        "sim/flightmodel/position/elevation".to_string(),
        serde_json::Value::Number(serde_json::Number::from_f64(2540.43).unwrap()),
    );
    app_state.plane_state.insert(
        "sim/flightmodel/position/indicated_airspeed".to_string(),
        serde_json::Value::Number(serde_json::Number::from_f64(100.).unwrap()),
    );

    match post_state(app_state.plane_state).await {
        Ok(()) => println!("Hello, world!"),
        Err(e) => print!("Error: {}", e),
    }
}

async fn post_state(plane_state: HashMap<String, Value>) -> Result<(), reqwest::Error> {
    let mut headers = HeaderMap::new();

    headers.insert(
        HeaderName::from_static("client-version"), // header name needs to be lowercase
        HeaderValue::from_static("2022-06-28"),
    );

    let body = json!({
            "plane_state" : plane_state,
    });

    println!("Json body: {}", body);

    let client = reqwest::Client::new();
    let resp = client
        .post("http://localhost:3000/api/v1/state")
        .headers(headers)
        .json(&body)
        .send()
        .await?;

    println!("status code: {}", resp.status());

    Ok(())
}
