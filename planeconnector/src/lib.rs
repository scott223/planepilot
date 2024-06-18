use std::time::Duration;

use crossterm::event::{Event, EventStream, KeyCode};

use futures::StreamExt;
use futures_timer::Delay;
use tokio::sync::mpsc;
use types::PlaneState;

use crate::types::AppState;

pub mod httpserver;
pub mod types;
pub mod utils;
pub mod xplanedatamap;
pub mod xplaneudp;

pub async fn run_app() -> anyhow::Result<()> {
    let (tx_command, rx_command) = mpsc::channel(32);
    let (tx_state, rx_state) = mpsc::channel(32);

    let plane_state = PlaneState::new(rx_state);
    let plane_state_proxy: types::PlaneStateProxy = types::PlaneStateProxy::new(tx_state);

    let app_state: AppState = AppState::new(tx_command, plane_state_proxy);

    tokio::select! {
        _ = plane_state.process() => {
    
        }
        _ = httpserver::run_server(&app_state) => { 
            
        }
        _ = xplaneudp::listen_to_xplane(&app_state) => { 
            
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
