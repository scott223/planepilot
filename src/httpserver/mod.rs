use anyhow::Context;
use axum::{
    extract::{Path, State},
    http::{Method, StatusCode},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use std::net::SocketAddr;
use tokio::net::TcpSocket;

use tower_http::cors::{Any, CorsLayer};
use tracing::{event, Level};

use crate::{types::Command, utils};

// define the routes and attach the state proxy, and serve the server
pub(super) async fn run_server(
    app_state: std::sync::Arc<std::sync::Mutex<crate::types::AppState>>,
) -> anyhow::Result<()> {
    let addr: SocketAddr = "127.0.0.1:3200"
        .parse()
        .context("cannot parse socket address")?;

    let cors = CorsLayer::new()
        // allow `GET` and `POST` when accessing the resource
        .allow_methods([Method::GET, Method::POST])
        // allow requests from any origin
        .allow_origin(Any);

    // build our application with the routes
    let app: Router = Router::new()
        .route("/", get(root))
        .route("/api/v1/autopilot_state", get(get_autopilot_state))
        .route("/api/v1/plane_state", get(get_plane_state))
        .route("/api/v1/activate/{direction}/{mode}", get(activate_mode))
        .route("/api/v1/set/{key}/{value}", get(set_key))
        .route("/api/v1/switch/{key}", get(switch_key))
        .route("/api/v1/command", post(send_command))
        .layer(utils::return_trace_layer())
        .layer(cors)
        .with_state(app_state);

    let socket = TcpSocket::new_v4().unwrap();
    socket.set_reuseaddr(true).unwrap(); // allow to reuse the addr both for connect and listen
    socket.set_reuseport(true).unwrap(); // same for the port
    socket
        .bind(addr)
        .context("Cannot bind autopilot HTTP server port")?;

    let listener = socket.listen(1024).context("Cannot start HTTP listener.")?;

    event!(
        Level::INFO,
        "Autopilot HTTP server started to listen on address {:?}",
        listener
            .local_addr()
            .context("Error getting local address for HTTP server")?
    );

    // serve the server
    axum::serve(listener, app)
        .await
        .context("Error serving HTTP server.")?;

    Ok(())
}

// basic handler that responds with a static string - can be used as a heart beat
async fn root() -> &'static str {
    "Hello, World!"
}

// get the current plane state from the app and serve as a JSON
async fn get_plane_state(
    State(app_state): State<std::sync::Arc<std::sync::Mutex<crate::types::AppState>>>,
) -> Result<impl axum::response::IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let state = app_state
        .lock()
        .expect("run_server: cannot get lock on app state");
    Ok(Json(state.plane_state.clone()))
}

// struct to receive commands over http
#[derive(Debug, Deserialize, Serialize)]
pub struct SendCommand {
    pub command: String,
    pub value: f64,
}

// receive a command and send a command message on the channel
async fn send_command(
    State(app_state): State<std::sync::Arc<std::sync::Mutex<crate::types::AppState>>>,
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

    let state = app_state.lock().expect("cannot get lock on state");

    // TODO actually send the command!

    //match state.command_sender.send(command).await() {
    //    Ok(_) => return Ok(StatusCode::OK),
    //    Err(e) => {
    //        event!(Level::ERROR, "Cannot send command: {:?}", e);
    return Ok(StatusCode::INTERNAL_SERVER_ERROR);
    //    }
    //};
}

// AUTOPILOT

// get the current autopilot state from the app and serve as a JSON
async fn get_autopilot_state(
    State(app_state): State<std::sync::Arc<std::sync::Mutex<crate::types::AppState>>>,
) -> Result<impl axum::response::IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let state = app_state
        .lock()
        .expect("run_server: cannot get lock on app state");
    Ok(Json(state.autopilot_state.clone()))
}

async fn switch_key(
    Path(key): Path<String>,
    State(app_state): State<std::sync::Arc<std::sync::Mutex<crate::types::AppState>>>,
) -> Result<impl axum::response::IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let mut state = app_state
        .lock()
        .expect("run_server: cannot get lock on app state");

    // TODO better error handling
    match key.as_str() {
        "heading" => state.autopilot_state.activate_standby_heading(),
        "altitude" => state.autopilot_state.activate_standby_altitude(),
        "velocity" => state.autopilot_state.activate_standby_velocity(),
        _ => {
            return Ok(StatusCode::BAD_REQUEST);
        }
    }

    event!(Level::INFO, "Activated standby value for {}", key);
    Ok(StatusCode::OK)
}

async fn set_key(
    Path((key, value)): Path<(String, usize)>,
    State(app_state): State<std::sync::Arc<std::sync::Mutex<crate::types::AppState>>>,
) -> Result<impl axum::response::IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let mut state = app_state
        .lock()
        .expect("run_server: cannot get lock on app state");

    //TODO better error handling
    match (key.as_str(), value) {
        ("heading", _) => state.autopilot_state.set_standby_heading(value as f64),
        ("altitude", _) => state.autopilot_state.set_standby_altitude(value as f64),
        ("velocity", _) => state.autopilot_state.set_standby_velocity(value as f64),
        (_, _) => {
            return Ok(StatusCode::BAD_REQUEST);
        }
    };

    event!(Level::INFO, "Standby value set ({}, {})", key, value);
    Ok(StatusCode::OK)
}

async fn activate_mode(
    Path((direction, mode)): Path<(String, String)>,
    State(app_state): State<std::sync::Arc<std::sync::Mutex<crate::types::AppState>>>,
) -> Result<impl axum::response::IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let mut state = app_state
        .lock()
        .expect("run_server: cannot get lock on app state");

    //TODO better error handling
    match (direction.as_str(), mode.as_str()) {
        ("horizontal", "standby") => state
            .autopilot_state
            .activate_horizontal_guidance_standby_mode(),
        ("horizontal", "wingslevel") => state
            .autopilot_state
            .activate_horizontal_guidance_wingslevel_mode(),
        ("horizontal", "heading") => state
            .autopilot_state
            .activate_horizontal_guidance_heading_mode(),
        ("vertical", "standby") => state
            .autopilot_state
            .activate_vertical_guidance_standby_mode(),
        ("vertical", "tecs") => state.autopilot_state.activate_vertical_guidance_tecs_mode(),
        (_, _) => {
            return Ok(StatusCode::BAD_REQUEST);
        }
    };

    event!(
        Level::INFO,
        "Autopilot mode activated ({}, {})",
        direction,
        mode
    );

    Ok(StatusCode::OK)
}
