use std::sync::{Arc, RwLock};

use axum::{
    extract::State, http::Method, routing::get, Json, Router, http::StatusCode,
};

use tower_http::cors::{Any, CorsLayer};
use tracing::{event, Level};

use crate::{utils, AppState};

pub async fn run_server(app_state: &Arc<RwLock<AppState>>) {
    let cors = CorsLayer::new()
    // allow `GET` and `POST` when accessing the resource
    .allow_methods([Method::GET, Method::POST])
    // allow requests from any origin
    .allow_origin(Any);

    // build our application with the routes
    let app: Router = Router::new()
        .route("/", get(root))
        .route("/api/v1/state", get(get_state))
        .layer(utils::return_trace_layer())
        .layer(cors)
        .with_state(app_state.clone());

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3100")
        .await
        .expect("cannot start listener. exiting.");

    event!(
        Level::INFO,
        "Server started to listen on address {:?}",
        listener
            .local_addr()
            .expect("error getting local addr. exiting.")
    );

    axum::serve(listener, app)
        .await
        .expect("Error serving app. Exiting.");

}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World!"
}

pub async fn get_state(
    State(app_state): State<Arc<RwLock<AppState>>>,
) -> Result<impl axum::response::IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    { // extra scope to make sure drop the lock
        let r = app_state.read().unwrap();
        let state = &r.plane_state;
        Ok(Json(state.clone()))
    }

    /* 
    
    match update_channels(&app_state.db).await {
        Ok(c) => return Ok(Json(c)),
        Err(e) => {
            let error_response = serde_json::json!({
                "status": "error",
                "message": format!("Database error: { }", e),
            });
            event!(Level::ERROR, "Database error { }", e);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
        }
    }

    */
}
