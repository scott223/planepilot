use std::{collections::HashMap, io::Error, time::Duration};

use crossterm::event::{self, Event, KeyCode};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde_json::{json, Value};
use tokio::sync::mpsc;
use tokio::{net::UdpSocket, time};

use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone)]
pub struct AppState {
    pub plane_state: HashMap<String, Value>,
}

const FLOAT_LEN: usize = 4;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let mut app_state = AppState {
        plane_state: HashMap::new(),
    };

    app_state.plane_state.insert(
        "sim/flightmodel/position/elevation".to_string(),
        serde_json::Value::Number(serde_json::Number::from_f64(2540.43).unwrap()),
    );
    app_state.plane_state.insert(
        "sim/flightmodel/position/indicated_airspeed".to_string(),
        serde_json::Value::Number(serde_json::Number::from_f64(100.).unwrap()),
    );

    let socket = UdpSocket::bind("127.0.0.1:49100").await.unwrap();
    // Create a tokio::mpsc channel to send and recevie the the shutdown signal across workers
    let (tx, mut rx) = mpsc::channel(32);

    // Step 1: Create a new CancellationToken
    let token = CancellationToken::new();

    // Step 2: Clone the token for use in another task
    let cloned_token_udp = token.clone();

    // Task 1 - Wait for token cancellation or a long time
    let udp_handle = tokio::spawn(async move {
        tokio::select! {
            // Step 3: Using cloned token to listen to cancellation requests
            _ = cloned_token_udp.cancelled() => {
                // The token was cancelled, task can shut down
            }
            _ = listen_to_xplane(socket) => {
                // Long work has completed
            }
        }
    });

    let cloned_token_data = token.clone();

    // Task 2 - Wait for token cancellation or a long time
    let data_logger_handle = tokio::spawn(async move {
        tokio::select! {
            // Step 3: Using cloned token to listen to cancellation requests
            _ = cloned_token_data.cancelled() => {
                // The token was cancelled, task can shut down
            }
            _ = print_ja() => {
                // Long work has completed
            }
        }
    });

    let cloned_token_terminal = token.clone();

    // Task 3 - Wait for token cancellation or a long time
    let terminal_handle = tokio::spawn(async move {
        tokio::select! {
            // Step 3: Using cloned token to listen to cancellation requests
            _ = cloned_token_terminal.cancelled() => {
                // The token was cancelled, task can shut down
            }
            _ = run_terminal(tx) => {
                // Long work has completed
            }
        }
    });

    // Spawn another tokio worker to handle the shutdown once a signal is received
    let _shutdown = tokio::spawn(async move {
        // Listen for shutdown signal
        while let Some(_shutdown_signal) = rx.recv().await { //wait untill we have a shutdown signal from on of the workers
        }
        token.cancel();
    });

    // Wait for all the workers to finish
    let _ = tokio::try_join!(udp_handle, data_logger_handle, terminal_handle)
        .expect("unable to join tasks");
}

async fn print_ja() -> () {
    let mut interval = time::interval(Duration::from_secs(1));
    loop {
        println! {"ja"};
        interval.tick().await;
    }
}

async fn run_terminal(shutdown_channel: mpsc::Sender<bool>) -> Result<(), Error> {
    //key detection
    // Running main loop
    'mainloop: loop {
        while event::poll(Duration::default())? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Esc | KeyCode::Char('q') => {
                        // User pressed ESC or 'q', breaking the main loop
                        shutdown_channel
                            .send(true)
                            .await
                            .expect("unable to send shutdown signal");
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

async fn listen_to_xplane(socket: UdpSocket) -> Result<(), Error> {
    let mut buf: [u8; 512] = [0; 512];
    loop {
        let (_len, _src) = socket.recv_from(&mut buf).await.unwrap();
        if buf[0..4] == [68, 65, 84, 65] {
            // DATA

            for sentence in buf[5..].chunks(36) {
                //start at 5 as there is 0 byte after DATA

                match sentence[0] {
                    17_u8 => {
                        // PITCH, ROLL, HEADING
                        let pitch: f32 = f32::from_le_bytes(
                            sentence[4..4 + FLOAT_LEN]
                                .try_into()
                                .expect("Needed 4 bytes for a float"),
                        );
                        let roll: f32 = f32::from_le_bytes(
                            sentence[8..8 + FLOAT_LEN]
                                .try_into()
                                .expect("Needed 4 bytes for a float"),
                        );
                        let heading: f32 = f32::from_le_bytes(
                            sentence[12..12 + FLOAT_LEN]
                                .try_into()
                                .expect("Needed 4 bytes for a float"),
                        );
                        println!("pitch: {}, roll: {}, heading: {}", pitch, roll, heading)
                    }
                    20_u8 => {
                        // LATITUDE, LONGITUDE, ALTITUDE
                        let latitude: f32 = f32::from_le_bytes(
                            sentence[4..4 + FLOAT_LEN]
                                .try_into()
                                .expect("Needed 4 bytes for a float"),
                        );
                        let longitude: f32 = f32::from_le_bytes(
                            sentence[8..8 + FLOAT_LEN]
                                .try_into()
                                .expect("Needed 4 bytes for a float"),
                        );
                        let altitude_msl: f32 = f32::from_le_bytes(
                            sentence[12..12 + FLOAT_LEN]
                                .try_into()
                                .expect("Needed 4 bytes for a float"),
                        );
                        let altitude_agl: f32 = f32::from_le_bytes(
                            sentence[16..16 + FLOAT_LEN]
                                .try_into()
                                .expect("Needed 4 bytes for a float"),
                        );
                        let on_runway: bool =
                            if sentence[20..20 + FLOAT_LEN] == 1.0_f32.to_le_bytes() {
                                true
                            } else {
                                false
                            };

                        //println!(
                        //    "latitude: {}, longitude: {}, altitude_msl: {}, altitude_agl: {}, on_runway: {} \n",
                        //    longitude, latitude, altitude_msl, altitude_agl, on_runway
                        //)
                    }
                    _ => {
                        // do nothing
                    }
                };
            }
        }
    }
}

async fn post_state(plane_state: HashMap<String, Value>) -> Result<(), reqwest::Error> {
    let mut headers = HeaderMap::new();

    headers.insert(
        HeaderName::from_static("client-version"), // header name needs to be lowercase
        HeaderValue::from_static("2022-06-28"),
    );

    let body = json!({
            "plane_state" : plane_state,
    });

    println!("Json body: {}", body);

    let client = reqwest::Client::new();
    let resp = client
        .post("http://localhost:3000/api/v1/state")
        .headers(headers)
        .json(&body)
        .send()
        .await?;

    println!("status code: {}", resp.status());

    Ok(())
}
