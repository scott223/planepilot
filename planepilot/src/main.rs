use std::collections::HashMap;

use crossterm::event::{Event, EventStream, KeyCode};
use futures::StreamExt;
use futures_timer::Delay;
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
    println!("Planepilot started");

    tokio::select! {
        _ = run_autopilot() => {

        }
        _ = run_terminal() => {
            println!("terminal completed");
        }
    }
}

async fn run_autopilot() -> anyhow::Result<()> {
    let mut app_state = AppState {
        flying: false,
        plane_state: HashMap::new(),
        horizontal_mode: HorizontalModes::WingsLevel,
    };

    loop {
        match update_state().await {
            Ok(state) => {
                app_state.flying = true;
                app_state.plane_state = state;
                //dbg!(&app_state.plane_state);
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
                    let roll: f64 = app_state.plane_state.get("roll").unwrap().as_f64().unwrap();
                    let roll_rate = app_state.plane_state.get("P").unwrap().as_f64().unwrap();

                    let p: f64 = 0.015;
                    let d: f64 = 0.02;

                    let aileron: f64 = -(roll * p + roll_rate * d);

                    println!(
                        "Wings level mode - roll [deg]: {:.4}, roll_rate [deg/s]: {:.4}, aileron [0-1]: {:.4}",
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

        let _ = tokio::time::sleep(Duration::from_millis(100)).await;
    }
    // Ok((s))
}

async fn run_terminal() -> anyhow::Result<()> {
    let mut reader = EventStream::new();

    loop {
        let delay = Delay::new(Duration::from_millis(1_000));

        tokio::select! {
            _ = delay => {
                //println!(".\r");
            },
            maybe_event = reader.next() => {
                match maybe_event {
                    Some(Ok(event)) => {
                        println!("Event::{:?}\r", event);

                        if event == Event::Key(KeyCode::Char('r').into()) {
                            match reset_position().await {
                                Ok(_) => { println!("position reset")},
                                Err(e) => { println!("position reset error: {}",e)}
                            }

                        }

                        if event == Event::Key(KeyCode::Char('q').into()) {
                            break;
                        }
                    }
                    Some(Err(e)) => println!("Error: {:?}\r", e),
                    None => break,
                }
            }
        };
    }

    Ok(())
}

async fn reset_position() -> anyhow::Result<()> {
    let client = reqwest::Client::new();

    let _res = match client
        .post("http://localhost:3100/api/v1/reset")
        .send()
        .await
    {
        Ok(_res) => return Ok(()),
        Err(e) => return Err(e.into()),
    };
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
