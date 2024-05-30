use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    time::Duration,
};

use crossterm::event::{Event, EventStream, KeyCode};

use futures::StreamExt;
use futures_timer::Delay;
use tokio::sync::mpsc;

use crate::types::{AppState, PlaneState};

pub mod httpserver;
pub mod types;
pub mod utils;
pub mod xplanedatamap;
pub mod xplaneudp;

pub async fn run_app() -> anyhow::Result<()> {
    let (tx_command, rx_command) = mpsc::channel(32);

    let app_state: AppState = AppState {
        plane_state: Arc::new(RwLock::new(PlaneState {
            map: HashMap::new(),
        })),
        tx_command,
    };

    let mut plane_state_clone = app_state.plane_state.clone();

    //let app_state_clone = app_state.clone(); //create a clone, in this case it will only create a pointer as its an Arc<Mutex

    tokio::select! {
        _ = xplaneudp::listen_to_xplane(&mut plane_state_clone) => { //this will be the only mutable reference
            // Long work has completed
        }
        _ = httpserver::run_server(&app_state) => { //immutable reference, using the cloned arc
            // http server has exited
        }
        _ = xplaneudp::listen_to_send_commands(rx_command) => {

        }
        _ = run_terminal() => {
            println!("terminal completed");
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
