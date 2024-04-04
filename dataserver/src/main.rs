use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tower_http::trace::TraceLayer;
use tracing::{event, Level};

use chrono::Utc;

use sqlx::{migrate::MigrateDatabase, Pool, Sqlite, SqlitePool};
const DB_URL: &str = "sqlite://sqlite.db";

#[derive(Debug, Clone)]
struct AppState {
    channels: Vec<Channel>,
    db: Pool<Sqlite>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
struct Channel {
    ChannelId: i64,
    ChannelName: String,
}

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    if !Sqlite::database_exists(DB_URL).await.unwrap_or(false) {
        event!(Level::INFO, "Creating database {}", DB_URL);
        match Sqlite::create_database(DB_URL).await {
            Ok(_) => event!(Level::INFO, "Create db success"),
            Err(error) => panic!("error: {}", error),
        }
    } else {
        println!("Database already exists");
    }

    let db = SqlitePool::connect(DB_URL)
        .await
        .expect("can't connect to database");

    //let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let migrations = std::path::Path::new(
        "/Users/scottbrugmans/Development/rust/planepilot/planepilot/dataserver/migrations/",
    );
    let migration_results = sqlx::migrate::Migrator::new(migrations)
        .await
        .unwrap()
        .run(&db)
        .await;

    match migration_results {
        Ok(_) => event!(Level::INFO, "Migration success"),
        Err(error) => {
            panic!("error: {}", error);
        }
    }

    let mut app_state = AppState {
        channels: vec![],
        db,
    };

    app_state.channels = update_channels(&app_state).await;

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        .route("/api/v1/data", post(add_data))
        .route("/api/v1/channel", post(add_channel))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(tower_http::trace::DefaultMakeSpan::new().include_headers(true))
                .on_request(tower_http::trace::DefaultOnRequest::new().level(Level::INFO))
                .on_response(
                    tower_http::trace::DefaultOnResponse::new()
                        .level(Level::INFO)
                        .latency_unit(tower_http::LatencyUnit::Micros),
                ), // on so on for `on_eos`, `on_body_chunk`, and `on_failure`
        )
        .with_state(app_state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    event!(
        Level::INFO,
        "Server started to listen on address {:?}",
        listener.local_addr().unwrap()
    );

    axum::serve(listener, app).await.unwrap();
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World!"
}

async fn update_channels(app_state: &AppState) -> Vec<Channel> {
    let channels: Vec<Channel> = sqlx::query_as!(Channel, r"select * from channels")
        .fetch_all(&app_state.db)
        .await
        .expect("error fetching channels");

    event!(Level::INFO, "Channels updated (n = {})", channels.len());

    channels
}

async fn add_data(
    // this argument tells axum to parse the request body
    // as JSON into a `AddData` type
    State(app_state): State<AppState>,
    Json(payload): Json<AddData>,
) -> StatusCode {
    // insert your application logic here
    let data = Data {
        value: payload.value,
        timestamp: payload.timestamp,
        channel: payload.channel,
    };

    if app_state
        .channels
        .iter()
        .filter(|c| c.ChannelId == data.channel)
        .count()
        != 1
    {
        return StatusCode::BAD_REQUEST;
    }

    let _result = sqlx::query(
        "INSERT INTO datapoints (CreationDate, ChannelId, DataPointValue) VALUES (?, ?, ?)",
    )
    .bind(data.timestamp)
    .bind(data.channel)
    .bind(data.value)
    .execute(&app_state.db)
    .await
    .expect("error when executing query");

    //println!("Query result: {:?}", result);

    // this will be converted into a JSON response
    // with a status code of `201 Created`
    (StatusCode::CREATED)
}

async fn add_channel(
    // this argument tells axum to parse the request body
    // as JSON into a `AddData` type
    State(mut app_state): State<AppState>,
    Json(payload): Json<AddChannel>,
) -> StatusCode {
    if !payload.name.is_empty() {
        let _result = sqlx::query("INSERT INTO channels (ChannelName) VALUES (?)")
            .bind(payload.name)
            .execute(&app_state.db)
            .await
            .expect("error when executing query");
        app_state.channels = update_channels(&app_state).await;
        StatusCode::CREATED
    } else {
        StatusCode::NO_CONTENT
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AddData {
    pub value: f64,
    pub timestamp: chrono::DateTime<Utc>,
    pub channel: i64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Data {
    pub value: f64,
    pub timestamp: chrono::DateTime<Utc>,
    pub channel: i64,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct AddChannel {
    pub name: String,
}
