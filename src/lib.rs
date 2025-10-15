use std::sync::{Arc, Mutex};

use crossterm::event::{Event, EventStream, KeyCode};
use futures::StreamExt;

pub mod dataconnector;
pub mod types;
pub mod utils;
pub mod xplane;

pub async fn run_app() -> anyhow::Result<()> {
    let app_state = Arc::new(Mutex::new(types::AppState::new()));

    tokio::select! {

        _ = xplane::listen_to_xplane(app_state.clone()) => {},
        _ = dataconnector::share_with_external(app_state.clone()) => {},

        // process that runs a terminal, that looks for input (eg "q" press)
        // this is the process that will run to completion and then the tokio::select will cancel the rest
        _ = run_terminal() => { }
    }

    Ok(())
}

// listents to terminal inputs, and breaks on "q"
async fn run_terminal() -> Result<(), ()> {
    let mut reader = EventStream::new();

    loop {
        tokio::select! {
            maybe_event = reader.next() => {
                match maybe_event {
                    Some(Ok(event)) => {
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

/*
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
*/
