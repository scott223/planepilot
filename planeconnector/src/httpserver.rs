use serde::{Deserialize, Serialize};

use axum::{
    extract::State,
    http::{Method, StatusCode},
    routing::{get, post},
    Json, Router,
};

use tower_http::cors::{Any, CorsLayer};
use tracing::{event, Level};

use crate::{types::Command, utils, AppState};

pub async fn run_server(app_state: &AppState) {
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
        .with_state(app_state.clone());

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3100")
        .await
        .expect("Cannot start listener. Exiting.");

    event!(
        Level::INFO,
        "Server started to listen on address {:?}",
        listener
            .local_addr()
            .expect("Error getting local address. Exiting.")
    );

    axum::serve(listener, app)
        .await
        .expect("Error serving app. Exiting.");
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World!"
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SendCommand {
    pub command: String,
    pub value: f64,
}

async fn send_command(
    State(app_state): State<AppState>,
    Json(payload): Json<SendCommand>,
) -> Result<impl axum::response::IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let command: Command = match payload.command.as_str() {
        "aileron" => Command::new_aileron(payload.value),
        "elevator" => Command::new_elevator(payload.value),
        "throttle" => Command::new_throttle(payload.value),
        _ => {
            return Ok(StatusCode::NOT_IMPLEMENTED);
        }
    };

    let _ = app_state
        .tx_command
        .send(command)
        .await
        .map_err(|e| event!(Level::ERROR, "Cannot send command: {:?}", e));

    Ok(StatusCode::OK)
}

pub async fn get_state(
    State(app_state): State<AppState>,
) -> Result<impl axum::response::IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    {
        // extra scope to make sure drop the lock
        let r = app_state.plane_state.read().unwrap();
        Ok(Json(r.map.clone()))
    }
}