use std::collections::HashMap;

use tokio::time::Duration;

use serde_json::{Number, Value};

pub struct AppState {
    flying: bool,
    plane_state: HashMap<String, Value>,
    horizontal_mode: HorizontalModes,
}

enum HorizontalModes {
    Standby,
    WingsLevel,
}

#[derive(Debug)]
enum SpecificErrors {
    PlaneConnectorNotReachable,
    PlaneConnectorReturnedError,
    StateNotUpdatedRecently,
}

impl std::fmt::Display for SpecificErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PlaneConnectorNotReachable => write!(f, "PlaneConnector is not reachable"),
            Self::PlaneConnectorReturnedError => write!(f, "PlaneConnector returned an error"),
            Self::StateNotUpdatedRecently => {
                write!(f, "State is available, but not updated")
            }
        }
    }
}

impl std::error::Error for SpecificErrors {}

#[tokio::main]
async fn main() {
    let mut app_state = AppState {
        flying: false,
        plane_state: HashMap::new(),
        horizontal_mode: HorizontalModes::WingsLevel,
    };

    println!("Planepilot started");

    loop {
        match update_state().await {
            Ok(state) => {
                app_state.flying = true;
                app_state.plane_state = state;
                dbg!(&app_state.plane_state);
            }
            Err(e) => {
                app_state.flying = false;
                app_state.plane_state.clear();
                println!("Error while updating state: {}", e);

                /*
                match e.downcast_ref() {
                    Some(SpecificErrors::PlaneConnectorNotReachable) => {
                        println!("1{:?}", e);
                    }
                    Some(SpecificErrors::StateNotUpdatedRecently) => {
                        println!("2{:?}", e);
                    }
                    Some(SpecificErrors::PlaneConnectorReturnedError) => {
                        println!("4{:?}", e);
                    }
                    None => {
                        println!("3");
                    }
                }

                */
            }
        };

        if app_state.flying {
            //horizontal mode

            match app_state.horizontal_mode {
                HorizontalModes::Standby => {
                    println!("Horizontal mode standby, no autopilot input for ailerons");
                }
                HorizontalModes::WingsLevel => {
                    let roll: &Value = app_state.plane_state.get("roll").unwrap();
                    let roll_rate = app_state.plane_state.get("P").unwrap();

                    let p: f64 = 3. / 60.0; // factor is 3, and dimension for 60 degrees roll = 1 aileron
                    let d: f64 = 4. / 90.0; // factor is, and dimension for 1 rad / s roll = 1 aileron

                    let aileron: f64 =
                        -(roll.as_f64().unwrap() * p + roll_rate.as_f64().unwrap() * d);

                    println!(
                        "roll: {}, roll_rate: {}, aileron: {}",
                        roll, roll_rate, aileron
                    );

                    let mut map: HashMap<String, Value> = HashMap::new();
                    map.insert("command".to_string(), Value::String("aileron".to_string()));
                    map.insert(
                        "value".to_string(),
                        Value::Number(Number::from_f64(aileron).unwrap()),
                    );

                    let client = reqwest::Client::new();

                    let _res = match client
                        .post("http://localhost:3100/api/v1/command")
                        .json(&map)
                        .send()
                        .await
                    {
                        Ok(_res) => {}
                        Err(_) => {}
                    };
                }
            }
        }

        let _ = tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

async fn update_state() -> anyhow::Result<HashMap<String, Value>> {
    let res = match reqwest::get("http://localhost:3100/api/v1/state").await {
        Ok(res) => res,
        Err(_) => {
            return Err(anyhow::Error::new(
                SpecificErrors::PlaneConnectorNotReachable,
            ))
        }
    };

    match res.status() {
        reqwest::StatusCode::OK => {
            let state = res.json::<HashMap<String, Value>>().await?;
            if !state.contains_key("last_updated_timestamp") {
                return Err(anyhow::Error::new(SpecificErrors::StateNotUpdatedRecently));
            }
            return Ok(state);
        }
        _ => {
            return Err(anyhow::Error::new(
                SpecificErrors::PlaneConnectorReturnedError,
            ))
        }
    }
}
