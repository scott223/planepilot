use std::{collections::HashMap, sync::{Arc, RwLock}, time::Duration};

use crossterm::event::{Event, KeyCode, EventStream};

use futures::StreamExt;
use futures_timer::Delay;

use crate::types::AppState;

pub mod utils;
pub mod httpserver;
pub mod xplanedatamap;
pub mod xplaneudp;
pub mod types;

pub async fn run_app() -> anyhow::Result<()> {
    let mut app_state: Arc<RwLock<AppState>> = Arc::new(RwLock::new(AppState {
        plane_state: HashMap::new(),
    }));

    let app_state_clone = app_state.clone(); //create a clone, in this case it will only create a pointer as its an Arc<Mutex
    
    tokio::select! {
        _ = xplaneudp::listen_to_xplane(&mut app_state) => { //this will be the only mutable reference
            // Long work has completed
        }
        _ = httpserver::run_server(&app_state_clone) => { //immutable reference, using the cloned arc
            // http server has exited
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
                println!(".\r"); 
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
