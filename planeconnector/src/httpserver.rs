use std::sync::Arc;

use axum::{
    http::Method,
    routing::get,
    Router,
};

use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};
use tracing::{event, Level};

use crate::{utils, AppState};

pub async fn run_server(app_state: &Arc<Mutex<AppState>>) {
    let cors = CorsLayer::new()
    // allow `GET` and `POST` when accessing the resource
    .allow_methods([Method::GET, Method::POST])
    // allow requests from any origin
    .allow_origin(Any);

    // build our application with the routes
    let app: Router = Router::new()
        .route("/", get(root))
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
