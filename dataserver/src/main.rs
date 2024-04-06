use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tower_http::trace::TraceLayer;
use tracing::{event, Level};

use chrono::{DateTime, Utc};

use sqlx::{migrate::MigrateDatabase, Pool, Sqlite, SqlitePool};
const DB_URL: &str = "sqlite://sqlite.db";

#[derive(Debug, Clone)]
struct AppState {
    channels: Vec<Channel>,
    db: Pool<Sqlite>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
struct Channel {
    channel_id: i64,
    channel_name: String,
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

    app_state.channels = update_channels(&app_state).await.unwrap();

    // build our application with a route
    let app = Router::new()
        .route("/", get(root))
        .route("/api/v1/data", post(add_data))
        .route("/api/v1/data/:channel_id", get(get_data))
        .route("/api/v1/channel", post(add_channel).get(get_channels))
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

async fn update_channels(app_state: &AppState) -> Result<Vec<Channel>, sqlx::Error> {
    match sqlx::query_as!(
        Channel,
        r"select ChannelId as channel_id, ChannelName as channel_name from channels"
    )
    .fetch_all(&app_state.db)
    .await
    {
        Ok(c) => {
            event!(Level::INFO, "Channels updated (n = {})", c.len());
            return Ok(c);
        }
        Err(e) => {
            event!(Level::ERROR, "Error updating channels: {})", e);
            return Err(e);
        }
    };
}

async fn get_data(
    State(app_state): State<AppState>,
) -> Result<impl axum::response::IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // sqlite does not have Utc, only NaiveDateTime, so need to do a typecast here
    match sqlx::query_as!(Data, r"select CreationDate as 'timestamp: DateTime<Utc>', DataPointValue as value, ChannelId as channel from datapoints")
        .fetch_all(&app_state.db)
        .await
    {
        Ok(d) => {
            return Ok(Json(d));
        }
        Err(e) => {
            event!(Level::ERROR, "Error when pulling data: {})", e);
            let error_response = serde_json::json!({
                "status": "error",
                "message": format!("Database error: {:}", e),
            });
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
        }
    };
}

async fn add_data(
    State(app_state): State<AppState>,
    Json(payload): Json<AddData>,
) -> Result<impl axum::response::IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // insert your application logic here
    let data = Data {
        value: payload.value,
        timestamp: payload.timestamp.unwrap_or(chrono::Utc::now()),
        channel: payload.channel,
    };

    if app_state
        .channels
        .iter()
        .filter(|c| c.channel_id == data.channel)
        .count()
        != 1
    {
        let error_response = serde_json::json!({
            "status": "error",
            "message": format!("Channel with this id does not exist"),
        });

        return Err((StatusCode::BAD_REQUEST, Json(error_response)));
    }

    let _result = sqlx::query(
        "INSERT INTO datapoints (CreationDate, ChannelId, DataPointValue) VALUES (?, ?, ?)",
    )
    .bind(data.timestamp)
    .bind(data.channel)
    .bind(data.value)
    .execute(&app_state.db)
    .await
    .map_err(|e| {
        let error_response = serde_json::json!({
            "status": "error",
            "message": format!("Database error: { }", e),
        });
        event!(Level::ERROR, "Database error { }", e);
        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
    })?;

    Ok((StatusCode::CREATED, Json(data)))
}

async fn add_channel(
    // this argument tells axum to parse the request body
    // as JSON into a `AddData` type
    State(mut app_state): State<AppState>,
    Json(payload): Json<AddChannel>,
) -> Result<impl axum::response::IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    if !payload.channel_name.is_empty() {
        let _result = sqlx::query("INSERT INTO channels (ChannelName) VALUES (?)")
            .bind(payload.channel_name)
            .execute(&app_state.db)
            .await
            .map_err(|e| {
                let error_response = serde_json::json!({
                    "status": "error",
                    "message": format!("Database error: { }", e),
                });
                event!(Level::ERROR, "Database error { }", e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
            })?;

        app_state.channels = update_channels(&app_state).await.unwrap();
        Ok((StatusCode::CREATED, Json(app_state.channels)))
    } else {
        let error_response = serde_json::json!({
            "status": "error",
            "message": format!("Channel name cannot be empty"),
        });
        Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
    }
}

async fn get_channels(
    State(app_state): State<AppState>,
) -> Result<impl axum::response::IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    return Ok(Json(update_channels(&app_state).await.unwrap()));
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AddData {
    pub value: f64,
    pub timestamp: Option<chrono::DateTime<Utc>>,
    pub channel: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct Data {
    pub value: f64,
    pub timestamp: chrono::DateTime<Utc>,
    pub channel: i64,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct AddChannel {
    pub channel_name: String,
}
