use std::time::Duration;

use crossterm::event::{Event, EventStream, KeyCode};

use futures::StreamExt;
use futures_timer::Delay;
use tokio::sync::mpsc;

use crate::types::{AppStateProxy, AppState};

pub mod httpserver;
pub mod types;
pub mod utils;
pub mod xplanedatamap;
pub mod xplaneudp;

pub async fn run_app() -> anyhow::Result<()> {
    let (tx_command, rx_command) = mpsc::channel(32);
    let (tx_state, rx_state) = mpsc::channel(32);

    let app_state: AppState = AppState::new(rx_state);
    let app_state_proxy: AppStateProxy = AppStateProxy::new(tx_state, tx_command);

    tokio::select! {
        _ = app_state.process() => {
        }
        _ = xplaneudp::listen_to_xplane(app_state_proxy.clone()) => { 
            
        }
        _ = httpserver::run_server(app_state_proxy.clone()) => { 
            
        }
        _ = xplaneudp::listen_to_send_commands(rx_command) => {

        }
        _ = run_terminal() => {

        }
    }

    Ok(())
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
