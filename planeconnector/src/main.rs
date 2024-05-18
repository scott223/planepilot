use core::panic;
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

    let _app_state = AppState {
        plane_state: HashMap::new(),
    };

    run_app().await;
}

async fn run_app() -> () {
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

fn translate_to_floats(data_bytes: [u8; 8 * FLOAT_LEN]) -> Result<Vec<f32>, Error> {
    let mut floats: Vec<f32> = Vec::with_capacity(8);

    for f in data_bytes.chunks(FLOAT_LEN) {
        floats.push(f32::from_le_bytes(
            f.try_into().expect("Need 4 bytes for a f32 float"),
        ));
    }

    Ok(floats)
}

async fn listen_to_xplane(socket: UdpSocket) -> Result<(), Error> {
    let mut buf: [u8; 1024] = [0_u8; 1024];

    loop {
        let (_len, _src) = socket
            .recv_from(&mut buf)
            .await
            .expect("Error whilst receiving UDP packet");

        if &buf[0..4] == b"DATA" {
            for sentence in buf[5..].chunks(36) {
                match sentence[0] {
                    17_u8 => {
                        let values = match translate_to_floats(
                            sentence[FLOAT_LEN..FLOAT_LEN + 8 * FLOAT_LEN]
                                .try_into()
                                .expect("need 32 bytes"), //start at byte index 4 (first four are used for xplane index)
                        ) {
                            Ok(v) => v,
                            Err(e) => panic!("error translating values: {}", e),
                        };
                        println!(
                            "pitch: {}, roll: {}, heading: {}",
                            values[0], values[1], values[2]
                        )
                    }
                    20_u8 => {
                        let values = match translate_to_floats(
                            sentence[FLOAT_LEN..FLOAT_LEN + 8 * FLOAT_LEN]
                                .try_into()
                                .expect("need 32 bytes"),
                        ) {
                            Ok(v) => v,
                            Err(e) => panic!("error translating values: {}", e),
                        };

                        let on_runway: bool = if values[4] == 1.0_f32 { true } else { false }; //convert a 1 to true

                        println!(
                            "latitude: {}, longitude: {}, altitude_msl: {}, altitude_agl: {}, on_runway: {} \n",
                            values[0], values[1], values[2], values[3], on_runway
                        )
                    }
                    _ => {
                        // do nothing
                    }
                };
            }
        }
    }
}

async fn _post_state(plane_state: HashMap<String, Value>) -> Result<(), reqwest::Error> {
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
