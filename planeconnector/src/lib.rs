use std::{collections::HashMap, sync::{Arc, RwLock}, time::Duration};

use crossterm::event::{self, Event, KeyCode};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::{sync::mpsc, time};
use tokio_util::sync::CancellationToken;

use anyhow::Result;

pub mod utils;
pub mod httpserver;
pub mod xplanedatamap;
pub mod xplaneudp;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppState {
    pub plane_state: HashMap<String, Value>,
}

#[derive(Debug)]
pub struct Command {
    command_type: CommandType,
    value: f64,
}

#[derive(Debug)]
enum CommandType {
    Throttle,
    Aileron,
    Elevator,
}

pub async fn run_app() -> Result<()> {
    let mut app_state: Arc<RwLock<AppState>> = Arc::new(RwLock::new(AppState {
        plane_state: HashMap::new(),
    }));

    let app_state_clone = app_state.clone(); //create a clone, in this case it will only create a pointer as its an Arc<Mutex

    // Create a tokio::mpsc channel to send and recevie the the shutdown signal across workers
    let (tx_shutdown, mut rx_shutdown) = mpsc::channel(32);
    let token = CancellationToken::new();

    // spwan a thread with all the main tasks, and a task to watch the cancellation token
    let cloned_token_udp = token.clone();
    let main_handle = tokio::spawn(async move {
        tokio::select! {
            // Step 3: Using cloned token to listen to cancellation requests
            _ = cloned_token_udp.cancelled() => {
                // The token was cancelled, task can shut down
            }
            _ = httpserver::run_server(&app_state_clone) => { //immutable reference
                // http server has exited
            }
            _ = xplaneudp::listen_to_xplane(&mut app_state) => { //this wlil be the only mutable reference
                // Long work has completed
            }
            _ = print_ja() => {
                // Long work has completed
            }
        }
    });

    // create a seperate task for the terminal, as this is currently blocking
    let cloned_token_terminal = token.clone();
    let terminal_handle = tokio::spawn(async move {
        tokio::select! {
            // Step 3: Using cloned token to listen to cancellation requests
            _ = cloned_token_terminal.cancelled() => {
                // The token was cancelled, task can shut down
            }
            _ = run_terminal(tx_shutdown) => {
                // Long work has completed
            }
        }
    });

    // Spawn another tokio worker to handle the shutdown once a signal is received
    let _shutdown = tokio::spawn(async move {
        // Listen for shutdown signal
        while let Some(_shutdown_signal) = rx_shutdown.recv().await { //wait untill we have a shutdown signal from on of the workers
        }
        token.cancel();
    });

    // Wait for all the workers to finish
    tokio::try_join!(main_handle, terminal_handle)?;

    Ok(())
}

async fn run_terminal(shutdown_channel: mpsc::Sender<bool>) -> Result<()> {
    //key detection - currently sync code...
    // Running main loop
    'mainloop: loop {
        while event::poll(Duration::default())? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Esc | KeyCode::Char('q') => {
                        // User pressed ESC or 'q', breaking the main loop
                        shutdown_channel.send(true).await?;
                        break 'mainloop;
                    }
                    KeyCode::Char('p') => {
                        // User pressed 'p', forcing an update of the events
                        println!("Paaaaasfasdfjahskjhadskjhasdkjf");
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

async fn print_ja() {
    let mut interval = time::interval(Duration::from_secs(1));
    loop {
        println! {"ja"};
        interval.tick().await;
    }
}
