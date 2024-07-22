use std::collections::HashMap;

use serde_json::{Number, Value};
use tokio::{sync::mpsc, time::Duration};
use tracing::{event, Level};
use types::*;

pub mod horizontalguidance;
pub mod httpserver;
pub mod types;
pub mod utils;
pub mod verticalguidance;

pub async fn run_app() -> anyhow::Result<()> {
    let (tx_state, rx_state) = mpsc::channel(8);

    // set up the app state and a proxy, that is linked through a channel. we can then clone and share the proxy with all the different procsesses
    let app_state: AppState = AppState::new(rx_state);
    let app_state_proxy: AppStateProxy = AppStateProxy::new(tx_state);

    tokio::select! {
        _ = app_state.process() => { }
        _ = run_autopilot(app_state_proxy.clone()) => { }
        _ = share_state_with_data_server(app_state_proxy.clone()) => { }
        _ = httpserver::run_server(app_state_proxy.clone()) => { }
    }

    event!(Level::INFO, "Planepilot closed");

    Ok(())
}

async fn run_autopilot(app_state_proxy: AppStateProxy) -> anyhow::Result<()> {
    const MILLISECONDS_PER_LOOP: u64 = 200;

    let mut local_error_state: bool = true;

    loop {
        match update_state().await {
            Ok(plane_state) => {
                app_state_proxy.set_plane_state(plane_state).await?;
                event!(Level::TRACE, "Plane state updated");

                if !local_error_state {
                    local_error_state = true;
                    app_state_proxy.set_flying(true).await?;
                    event!(
                        Level::INFO,
                        "Connection to planeconnector achieved and state updated"
                    );
                }
            }
            Err(e) => {
                if local_error_state {
                    local_error_state = false;
                    app_state_proxy.set_flying(false).await?;
                    app_state_proxy.clear_plane_state().await?;
                    app_state_proxy.activate_vertical_standby_mode().await?;
                    app_state_proxy.activate_horizontal_standby_mode().await?;

                    event!(
                        Level::ERROR,
                        "Error when updating state so autopilot set to standby: {:?}",
                        e
                    );
                }
            }
        };

        let auto_pilot_state: types::AutoPilotState =
            app_state_proxy.get_auto_pilot_state().await?;

        // refresh the constants now every cycle, to iterate fast
        app_state_proxy.refresh_autopilot_constants().await?;

        if auto_pilot_state.are_we_flying {
            let client: reqwest::Client = reqwest::Client::new();

            let plane_state_struct: PlaneStateStruct =
                app_state_proxy.get_plane_state_as_struct().await?;

            let dt: f64 = (MILLISECONDS_PER_LOOP as f64 / 1000.0);

            verticalguidance::execute_vertical_guidance(
                dt,
                &client,
                &app_state_proxy,
                &auto_pilot_state,
                &plane_state_struct,
            )
            .await?;

            horizontalguidance::execute_horizontal_guidance(
                dt,
                &client,
                &app_state_proxy,
                &auto_pilot_state,
                &plane_state_struct,
            )
            .await?
        }

        let _ = tokio::time::sleep(Duration::from_millis(MILLISECONDS_PER_LOOP)).await;
    }
}

async fn send_command(
    client: &reqwest::Client,
    command_type: types::CommandType,
    value: f64,
) -> anyhow::Result<()> {
    let mut map: HashMap<String, Value> = HashMap::new();

    match command_type {
        CommandType::Aileron => {
            map.insert("command".to_string(), Value::String("aileron".to_string()));
        }
        CommandType::Elevator => {
            map.insert("command".to_string(), Value::String("elevator".to_string()));
        }
        CommandType::Throttle => {
            map.insert("command".to_string(), Value::String("throttle".to_string()));
        }
    }

    map.insert(
        "value".to_string(),
        Value::Number(Number::from_f64(value).unwrap()),
    );

    let _res = match client
        .post("http://localhost:3100/api/v1/command")
        .json(&map)
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

async fn share_state_with_data_server(app_state_proxy: AppStateProxy) -> anyhow::Result<()> {
    let client = reqwest::Client::new();

    loop {
        let state = app_state_proxy.get_auto_pilot_state().await?;
        if state.are_we_flying {
            let mut jsons: Vec<Value> = Vec::with_capacity(3);

            jsons.push(serde_json::json!({
                    "state_type": "AutoPilotState",
                    "state": state.horizontal_control_metrics,
            }));

            jsons.push(serde_json::json!({
                    "state_type": "AutoPilotState",
                    "state": state.horizontal_guidance,
            }));

            jsons.push(serde_json::json!({
                    "state_type": "AutoPilotState",
                    "state": state.vertical_guidance,
            }));

            for j in jsons {
                let _res = match client
                    .post("http://localhost:3000/api/v1/state")
                    .json(&j)
                    .send()
                    .await
                {
                    Ok(_res) => {}
                    Err(e) => return Err(e.into()),
                };
            }
        };

        let _ = tokio::time::sleep(Duration::from_millis(1000)).await;
    }
}
