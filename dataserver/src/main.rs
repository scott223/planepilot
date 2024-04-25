use std::sync::Arc;

use axum::{
    http::Method,
    response::sse::Event as SseEvent,
    routing::{get, post},
    Router,
};
use dataserver::{controller, utils};
use tokio::sync::{broadcast, Mutex};

use dotenv::dotenv;

use sqlx::SqlitePool;
use tracing::{event, Level};

use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() {
    dotenv().ok();
    utils::trace::start_tracing_subscriber();

    let config = utils::Config::default();
    let db: SqlitePool = utils::db::create_and_migrate_db(&config).await;

    let (tx, _) = broadcast::channel::<SseEvent>(100);

    let cors = CorsLayer::new()
        // allow `GET` and `POST` when accessing the resource
        .allow_methods([Method::GET, Method::POST])
        // allow requests from any origin
        .allow_origin(Any);

    let app_state: controller::AppState = controller::AppState {
        channels: Arc::new(Mutex::new(
            controller::update_channels(&db)
                .await
                .expect("error updating channels. exiting."),
        )),
        db,
        config,
        tx,
    };

    utils::log::logo();

    // build our application with the routes
    let app: Router = Router::new()
        .route("/", get(root))
        .route(
            "/api/v1/channel",
            post(controller::add_channel).get(controller::get_channels),
        )
        .route(
            "/api/v1/data",
            get(controller::get_all_data).post(controller::add_data),
        )
        .route(
            "/api/v1/channel/:channel_id/data",
            get(controller::get_data),
        )
        .route(
            "/api/v1/channel/:channel_id/stream",
            get(dataserver::sse::sse::sse_handler_single_channel),
        )
        .route(
            "/api/v1/stream",
            get(dataserver::sse::sse::sse_handler_general_channel),
        )
        .layer(utils::trace::return_trace_layer())
        .layer(cors)
        .with_state(app_state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
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
        .with_graceful_shutdown(utils::log::shutdown_signal())
        .await
        .expect("error serving app. exiting.");
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World!"
}
