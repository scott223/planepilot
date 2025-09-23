#![warn(unused_extern_crates)]

use axum::{
    http::Method,
    response::sse::Event as SseEvent,
    routing::{get, post},
    Router,
};
use tokio::sync::broadcast;

use sqlx::SqlitePool;
use tracing::{event, Level};

use tower_http::cors::{Any, CorsLayer};

use std::net::SocketAddr;
use tokio::net::TcpSocket;

pub mod controller;
pub mod models;
pub mod sse;
pub mod utils;

pub async fn run_app(_service_adresses: &(String, String, String)) -> anyhow::Result<()> {
    let config = utils::Config::default();
    let db: SqlitePool = utils::db::create_and_migrate_db(&config).await;

    let (tx, _) = broadcast::channel::<SseEvent>(100);

    let cors = CorsLayer::new()
        // allow `GET` and `POST` when accessing the resource
        .allow_methods([Method::GET, Method::POST])
        // allow requests from any origin
        .allow_origin(Any);

    let app_state: controller::AppState = controller::AppState { db, config, tx };

    // build our application with the routes
    let app: Router = Router::new()
        .route("/", get(root))
        .route("/api/v1/channels", get(controller::get_channels))
        .route(
            "/api/v1/data",
            get(controller::get_all_data).post(controller::add_data),
        )
        .route("/api/v1/state", post(controller::add_state))
        .layer(utils::trace::return_trace_layer())
        .layer(cors)
        .with_state(app_state);

    // run our app with hyper, listening globally on port 3000
    //let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
    //    .await
    //    .expect("cannot start listener. exiting.");

    let addr: SocketAddr = "0.0.0.0:3000".parse().unwrap();

    let socket = TcpSocket::new_v4().unwrap();
    socket.set_reuseaddr(true).unwrap(); // allow to reuse the addr both for connect and listen
    socket.set_reuseport(true).unwrap(); // same for the port
    socket.bind(addr).expect("cannot bind dataserver port");

    let listener = socket.listen(1024).expect("cannot start listener. exiting.");

    event!(
        Level::INFO,
        "pp_dataserver started, listening on address {:?}",
        listener
            .local_addr()
            .expect("error getting local addr. exiting.")
    );

    axum::serve(listener, app)
        .with_graceful_shutdown(utils::log::shutdown_signal())
        .await
        .expect("error serving app. exiting.");

    Ok(())
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World!"
}
