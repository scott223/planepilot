use axum::{
    extract::{Path, State},
    http::{Method, StatusCode},
    routing::get,
    Json, Router,
};

use tower_http::cors::{Any, CorsLayer};
use tracing::{event, Level};

use crate::{types::AppStateProxy, utils};

// define the routes and attach the state proxy, and serve the server
pub async fn run_server(app_state_proxy: AppStateProxy) {
    let cors = CorsLayer::new()
        // allow `GET` and `POST` when accessing the resource
        .allow_methods([Method::GET, Method::POST])
        // allow requests from any origin
        .allow_origin(Any);

    // build our application with the routes
    let app: Router = Router::new()
        .route("/", get(root))
        .route("/api/v1/state", get(get_autopilot_state))
        .route("/api/v1/activate/:direction/:mode", get(activate_mode))
        .layer(utils::return_trace_layer())
        .layer(cors)
        .with_state(app_state_proxy);

    // run our app with hyper, listening globally on port 3200
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3200")
        .await
        .expect("Cannot start listener. Exiting.");

    println!("hey");

    event!(
        Level::INFO,
        "Server started to listen on address {:?}",
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
pub async fn get_autopilot_state(
    State(app_state_proxy): State<AppStateProxy>,
) -> Result<impl axum::response::IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let state: crate::types::AutoPilotState = app_state_proxy
        .get_auto_pilot_state()
        .await
        .expect("error getting the state");

    Ok(Json(state))
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
            return Ok(StatusCode::OK);
        }
        Err(e) => {
            event!(Level::ERROR, "Cannot set autopilot mode: {:?}", e);
            return Ok(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
}
