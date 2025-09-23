use axum::{
    extract::{Path, State},
    http::{Method, StatusCode},
    routing::get,
    Json, Router,
};

use std::net::SocketAddr;
use tokio::net::TcpSocket;

use tower_http::cors::{Any, CorsLayer};
use tracing::{event, Level};

use super::{types::AppStateProxy, utils};

// define the routes and attach the state proxy, and serve the server
pub(super) async fn run_server(app_state_proxy: AppStateProxy) {
    let cors = CorsLayer::new()
        // allow `GET` and `POST` when accessing the resource
        .allow_methods([Method::GET, Method::POST])
        // allow requests from any origin
        .allow_origin(Any);

    // build our application with the routes
    let app: Router = Router::new()
        .route("/", get(root))
        .route("/api/v1/autopilot_state", get(get_autopilot_state))
        .route("/api/v1/activate/{direction}/{mode}", get(activate_mode))
        .route("/api/v1/set/{key}/{value}", get(set_key))
        .route("/api/v1/switch/{key}", get(switch_key))
        .layer(utils::return_trace_layer())
        .layer(cors)
        .with_state(app_state_proxy);

    // run our app with hyper, listening globally on port 3200
    //let listener = tokio::net::TcpListener::bind("0.0.0.0:3200")
    //    .await
    //    .expect("Cannot start listener. Exiting.");

    let addr: SocketAddr = "127.0.0.1:3200".parse().unwrap();

    let socket = TcpSocket::new_v4().unwrap();
    socket.set_reuseaddr(true).unwrap(); // allow to reuse the addr both for connect and listen
    socket.set_reuseport(true).unwrap(); // same for the port
    socket.bind(addr).expect("cannot bind autopilot port");

    let listener = socket
        .listen(1024)
        .expect("cannot start listener. exiting.");

    event!(
        Level::INFO,
        "pp_autopilot server started to listen on address {:?}",
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

// get the current autopilot state from the app and serve as a JSON
async fn get_autopilot_state(
    State(app_state_proxy): State<AppStateProxy>,
) -> Result<impl axum::response::IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let state: crate::types::AutoPilotState = app_state_proxy
        .get_auto_pilot_state()
        .await
        .expect("error getting the state");

    Ok(Json(state))
}

async fn switch_key(
    Path(key): Path<String>,
    State(app_state_proxy): State<AppStateProxy>,
) -> Result<impl axum::response::IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let res = match key.as_str() {
        "heading" => app_state_proxy.activate_heading_setpoint().await,
        "altitude" => app_state_proxy.activate_altitude_setpoint().await,
        "velocity" => app_state_proxy.activate_velocity_setpoint().await,
        _ => {
            return Ok(StatusCode::BAD_REQUEST);
        }
    };

    match res {
        Ok(_) => {
            event!(Level::INFO, "Activated value for {}", key);
            Ok(StatusCode::OK)
        }
        Err(e) => {
            event!(Level::ERROR, "Cannot activate value for: {:?}", e);
            Ok(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn set_key(
    Path((key, value)): Path<(String, usize)>,
    State(app_state_proxy): State<AppStateProxy>,
) -> Result<impl axum::response::IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let res = match (key.as_str(), value) {
        ("heading", _) => app_state_proxy.set_heading_standby(value as f64).await,
        ("altitude", _) => app_state_proxy.set_altitude_standby(value as f64).await,
        ("velocity", _) => app_state_proxy.set_velocity_standby(value as f64).await,
        (_, _) => {
            return Ok(StatusCode::BAD_REQUEST);
        }
    };

    match res {
        Ok(_) => {
            event!(Level::INFO, "Standby value set ({}, {})", key, value);
            Ok(StatusCode::OK)
        }
        Err(e) => {
            event!(Level::ERROR, "Cannot set value: {:?}", e);
            Ok(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn activate_mode(
    Path((direction, mode)): Path<(String, String)>,
    State(app_state_proxy): State<AppStateProxy>,
) -> Result<impl axum::response::IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let res = match (direction.as_str(), mode.as_str()) {
        ("horizontal", "standby") => app_state_proxy.activate_horizontal_standby_mode().await,
        ("horizontal", "wingslevel") => app_state_proxy.activate_horizontal_wingslevel_mode().await,
        ("horizontal", "heading") => app_state_proxy.activate_horizontal_heading_mode().await,
        ("vertical", "standby") => app_state_proxy.activate_vertical_standby_mode().await,
        ("vertical", "tecs") => app_state_proxy.activate_vertical_TECS_mode().await,
        (_, _) => {
            return Ok(StatusCode::BAD_REQUEST);
        }
    };

    match res {
        Ok(_) => {
            event!(
                Level::INFO,
                "Autopilot mode activated ({}, {})",
                direction,
                mode
            );
            Ok(StatusCode::OK)
        }
        Err(e) => {
            event!(Level::ERROR, "Cannot set autopilot mode: {:?}", e);
            Ok(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
