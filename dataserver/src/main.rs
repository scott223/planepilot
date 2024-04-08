use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router,
};
use dataserver::{controller, utils};
use tokio::sync::Mutex;

use dotenv::dotenv;

use sqlx::SqlitePool;
use tracing::{event, Level};

#[tokio::main]
async fn main() {
    dotenv().ok();
    let config: utils::Config = utils::Config::default();
    utils::trace::start_tracing_subscriber();
    let db: SqlitePool = utils::db::create_and_migrate_db(&config).await;

    utils::log::logo();

    let app_state: controller::AppState = controller::AppState {
        channels: Arc::new(Mutex::new(
            controller::update_channels(&db)
                .await
                .expect("error updating channels"),
        )),
        db,
    };

    // build our application with the routes
    let app: Router = Router::new()
        .route("/", get(root))
        .route("/api/v1/data", post(controller::add_data))
        .route("/api/v1/data/:channel_id", get(controller::get_data))
        .route(
            "/api/v1/channel",
            post(controller::add_channel).get(controller::get_channels),
        )
        .layer(utils::trace::return_trace_layer())
        .with_state(app_state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    event!(
        Level::INFO,
        "Server started to listen on address {:?}",
        listener.local_addr().expect("error getting local addr")
    );

    axum::serve(listener, app).await.expect("error serving app");
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World!"
}
