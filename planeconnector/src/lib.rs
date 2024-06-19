use std::time::Duration;

use crossterm::event::{Event, EventStream, KeyCode};

use futures::StreamExt;
use futures_timer::Delay;
use tokio::sync::mpsc;

use crate::types::{AppState, AppStateProxy};

pub mod httpserver;
pub mod types;
pub mod utils;
pub mod xplanedatamap;
pub mod xplaneudp;

pub async fn run_app() -> anyhow::Result<()> {
    // set up a channel for xplane commands, and state signals
    let (tx_command, rx_command) = mpsc::channel(32);
    let (tx_state, rx_state) = mpsc::channel(32);

    // set up the app state and a proxy, that is linked through a channel. we can then clone and share the proxy with all the different procsesses
    let app_state: AppState = AppState::new(rx_state);
    let app_state_proxy: AppStateProxy = AppStateProxy::new(tx_state, tx_command);

    tokio::select! {

        // process that runs on the app state, that will listen to the signals from the proxy and processes these
        _ = app_state.process() => { }

        // process that listens to xplane udp packets, and updatates the state accordingly
        _ = xplaneudp::listen_to_xplane(app_state_proxy.clone()) => { }

        // process that runs an http server, to share state and receive commands from the autopilot
        _ = httpserver::run_server(app_state_proxy.clone()) => { }

        // process that listens to incomming commands (through the http server), and send them to xplane
        _ = xplaneudp::listen_to_send_commands(rx_command) => { }

        // process that runs a terminal, that looks for input (eg "q" press)
        // this is the process that will run to completion and then the tokio::select will cancel the rest
        _ = run_terminal() => { }
    }

    Ok(())
}

// listents to terminal inputs, and breaks on "q"

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
