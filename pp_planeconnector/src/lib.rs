#![warn(unused_extern_crates)]

use std::time::Duration;

use tokio::sync::mpsc;

use self::types::{AppState, AppStateProxy};

pub mod httpserver;
pub mod types;
pub mod utils;
pub mod xplanedatamap;
pub mod xplaneudp;

pub async fn run_app(service_adresses: &(String, String, String)) -> anyhow::Result<()> {
    // set up a channel for xplane commands, and state signals
    let (tx_command, rx_command) = mpsc::channel(32);
    let (tx_state, rx_state) = mpsc::channel(32);

    // set up the app state and a proxy, that is linked through a channel. we can then clone and share the proxy with all the different procsesses
    let app_state: AppState = AppState::new(rx_state);
    let app_state_proxy: AppStateProxy = AppStateProxy::new(service_adresses, tx_state, tx_command);

    tokio::select! {

        // process that runs on the app state, that will listen to the signals from the proxy and processes these
        _ = app_state.process() => { }

        // process that listens to xplane udp packets, and updatates the state accordingly
        _ = xplaneudp::listen_to_xplane(app_state_proxy.clone()) => { }

        // process that listens to incomming commands (through the http server), and send them to xplane
        _ = xplaneudp::listen_to_send_commands(rx_command) => { }

        // process that runs an http server, to share state and receive commands from the autopilot
        _ = httpserver::run_server(app_state_proxy.clone()) => { }

        _ = share_state_with_data_server(app_state_proxy.clone()) => { }
    }

    Ok(())
}

async fn share_state_with_data_server(app_state_proxy: AppStateProxy) -> anyhow::Result<()> {
    let client = reqwest::Client::new();

    loop {
        let state = app_state_proxy.get_state().await?;

        if state.contains_key("last_updated_timestamp") {
            let json = &serde_json::json!({
                    "state_type": "PlaneState",
                    "state": state,

            });

            match client
                .post(app_state_proxy.service_adresses.0.to_owned() + "/state")
                .json(json)
                .send()
                .await
            {
                Ok(_res) => {}
                Err(e) => return Err(e.into()),
            }
        }

        let _ = tokio::time::sleep(Duration::from_millis(1000)).await;
    }
}
