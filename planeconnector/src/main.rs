use core::panic;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde_json::{json, Value};
use std::collections::HashMap;

use anyhow::Result;

#[derive(Debug, Clone)]
pub struct AppState {
    pub plane_state: HashMap<String, Value>,
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let _app_state = AppState {
        plane_state: HashMap::new(),
    };

    match planeconnector::run_app().await {
        Ok(()) => {}
        Err(e) => panic!("Error in main program: {}", e),
    }
}

async fn _post_state(plane_state: HashMap<String, Value>) -> Result<(), reqwest::Error> {
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
