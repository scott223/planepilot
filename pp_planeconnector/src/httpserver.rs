use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use axum::{
    extract::State,
    http::{Method, StatusCode},
    routing::{get, post},
    Json, Router,
};

use tower_http::cors::{Any, CorsLayer};
use tracing::{event, Level};

use super::{
    types::{AppStateProxy, Command},
    utils,
};

// define the routes and attach the state proxy, and serve the server
pub(super) async fn run_server(app_state: AppStateProxy) {
    let cors = CorsLayer::new()
        // allow `GET` and `POST` when accessing the resource
        .allow_methods([Method::GET, Method::POST])
        // allow requests from any origin
        .allow_origin(Any);

    // build our application with the routes
    let app: Router = Router::new()
        .route("/", get(root))
        .route("/api/v1/state", get(get_state))
        .route("/api/v1/command", post(send_command))
        .layer(utils::return_trace_layer())
        .layer(cors)
        .with_state(app_state);

    // run our app with hyper, listening globally on port 3100
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3100")
        .await
        .expect("Cannot start listener. Exiting.");

    event!(
        Level::INFO,
        "pp_planeconector server started to listen on address {:?}",
        listener
            .local_addr()
            .expect("Error getting local address. Exiting.")
    );

    // serve the server
    axum::serve(listener, app)
        .await
        .expect("Error serving app. Exiting.");
}

// basic handler that responds with a static string - can be used as a heart beat
async fn root() -> &'static str {
    "Hello, World!"
}

// struct to receive commands over http
#[derive(Debug, Deserialize, Serialize)]
pub struct SendCommand {
    pub command: String,
    pub value: f64,
}

// receive a command and send a command message on the channel
async fn send_command(
    State(app_state_proxy): State<AppStateProxy>,
    Json(payload): Json<SendCommand>,
) -> Result<impl axum::response::IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // create the command based on the incoming json
    let command: Command = match payload.command.as_str() {
        "aileron" => Command::new_aileron(payload.value),
        "elevator" => Command::new_elevator(payload.value),
        "throttle" => Command::new_throttle(payload.value),
        "reset" => Command::new_reset(),
        _ => {
            return Ok(StatusCode::NOT_IMPLEMENTED);
        }
    };

    // send the message
    match app_state_proxy.command_sender.send(command).await {
        Ok(_) => return Ok(StatusCode::OK),
        Err(e) => {
            event!(Level::ERROR, "Cannot send command: {:?}", e);
            return Ok(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }
}

// get the current state from the app and serve as a JSON
async fn get_state(
    State(app_state_proxy): State<AppStateProxy>,
) -> Result<impl axum::response::IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let state: HashMap<String, serde_json::Value> = app_state_proxy
        .get_state()
        .await
        .expect("error getting the state");

    Ok(Json(state))
}
