use anyhow::anyhow;
use serde_json::Value;
use std::{collections::HashMap, time::Duration};

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::sse::Event as SseEvent,
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqliteRow, FromRow, Pool, Row, Sqlite};

use tokio::sync::broadcast;
use tracing::{event, Level};

use super::{models, utils::Config};

use itertools::Itertools;

/// Holds the full app state, mostly db, config and a tx for the SSE broadcasts
#[derive(Debug, Clone)]
pub struct AppState {
    pub db: Pool<Sqlite>,
    pub config: Config,
    pub tx: broadcast::Sender<SseEvent>, //for the SSE broadcasts
}

#[derive(Debug, Deserialize)]
pub struct GetDataParams {
    // frame duration and offset are Optional, revert to defaults in config if needed
    frame_duration: Option<i32>,
    frame_end_offset: Option<i32>,
}

/// Gets all the data
/// using a standard frame, but you can also specify the frame duraction and end offset to get a customized frame

pub async fn get_all_data(
    State(app_state): State<AppState>,
    Query(params): Query<GetDataParams>,
) -> Result<impl axum::response::IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // define the time frame for which we are looking for data - use the defaults if no params are given
    let frame_duration: i32 = params
        .frame_duration
        .unwrap_or(app_state.config.data_frame_duration);

    let frame_end_offset: i32 = params
        .frame_end_offset
        .unwrap_or(app_state.config.data_frame_offset);

    let frame_end = Utc::now() - Duration::from_secs((frame_end_offset * 60) as u64);
    let frame_start = frame_end - Duration::from_secs((frame_duration * 60) as u64);

    match sqlx::query(r"select * from datapoints WHERE CreationDate BETWEEN (?) AND (?) ORDER BY ChannelName, CreationDate LIMIT 3600")
        .bind(frame_start)
        .bind(frame_end)
        // map to Data struct
        .try_map(|d| models::Data::from_row(&d))
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

/// Struct to add a full state at once

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AddState {
    pub plane_state: HashMap<String, Value>,
}

/// Handler to add a state Json to the database - will loop through the state hashmap and add each item

pub async fn add_state(
    State(app_state): State<AppState>,
    Json(payload): Json<AddState>,
) -> Result<impl axum::response::IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let timestamp = chrono::Utc::now().timestamp_nanos_opt().unwrap();

    let mut line: String = "plane_state ".to_owned();

    line.push_str(
        &payload
            .plane_state
            .iter()
            .filter(|(k, v)| v.is_number())
            .map(|(k, v)| format!("{}={}", k, v.as_f64().unwrap()))
            .join(","),
    );

    line.push_str(" ");
    line.push_str(&timestamp.to_string());

    dbg!(line.clone());

    let params = [("bucket", "Planepilot")];
    let client = reqwest::Client::new();
    let res = client
    .post("https://eu-central-1-1.aws.cloud2.influxdata.com/api/v2/write/?bucket=Planepilot")
    .header(reqwest::header::AUTHORIZATION, "Token fNYYFLGey5QUItHsf98hMZqJiB9f_FjzJZlPqih3UfD1QlXRy2AA4MU4p3UnxCBWXx90_FMKvHWoEmMALiQ_ew==")
    .body(line.to_string())
    .send()
    .await;

    match res {
        Ok(r) => {
            println!("ok {:?}", r.text().await);
        }
        Err(e) => {
            println! {"e: {}", e}
        }
    }

    //line += &timestamp.timestamp().to_string();

    //dbg!(line);

    Ok(StatusCode::OK)
}

/// Function to actually add a single data item to the database

pub async fn add_single_data(d: AddData, app_state: &AppState) -> anyhow::Result<()> {
    let _result = sqlx::query(
        "INSERT INTO datapoints (CreationDate, ChannelName, DataPointValue) VALUES (?, ?, ?)",
    )
    .bind(d.timestamp.unwrap_or(chrono::Utc::now()))
    .bind(d.channel)
    .bind(d.value)
    .execute(&app_state.db)
    .await
    .map_err(|e| {
        return anyhow!(e);
    });

    Ok(())
}

/// Struct to define new data

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AddData {
    pub value: f64,
    pub timestamp: Option<chrono::DateTime<Utc>>,
    pub channel: String,
}

/// Handle to process adding single data item

pub async fn add_data(
    State(app_state): State<AppState>,
    Json(payload): Json<AddData>,
) -> Result<impl axum::response::IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let _result = add_single_data(payload, &app_state).await.map_err(|e| {
        let error_response = serde_json::json!({
            "status": "error",
            "message": format!("Database error: { }", e),
        });
        event!(Level::ERROR, "Database error { }", e);
        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
    });

    Ok(StatusCode::CREATED)
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Channel {
    channel_name: String,
}

// implements the cast from Sqliterow to Data
// will need to write implementation for Postgres as well
impl<'r> FromRow<'r, SqliteRow> for Channel {
    fn from_row(row: &'r SqliteRow) -> Result<Self, sqlx::Error> {
        let channel_name = row.try_get("ChannelName")?;
        Ok(Channel { channel_name })
    }
}

/// Gets all channels currently in the db by using distinct select

pub async fn get_channels(
    State(app_state): State<AppState>,
) -> Result<impl axum::response::IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    match sqlx::query(r"select distinct ChannelName from datapoints")
        // map to Data struct
        .try_map(|c| Channel::from_row(&c))
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
