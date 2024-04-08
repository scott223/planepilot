use std::sync::Arc;
use tokio::sync::Mutex;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Pool, Sqlite};
use tracing::{event, Level};

use crate::models;

#[derive(Debug, Clone)]
pub struct AppState {
    pub channels: Arc<Mutex<Vec<models::Channel>>>,
    pub db: Pool<Sqlite>,
}

pub async fn update_channels(db: &Pool<Sqlite>) -> Result<Vec<models::Channel>, sqlx::Error> {
    match sqlx::query(r"select * from channels")
        .try_map(|d| models::Channel::from_row(&d))
        .fetch_all(db)
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
pub async fn get_data(
    State(app_state): State<AppState>,
    Path(channel_id): Path<i64>,
) -> Result<impl axum::response::IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    match sqlx::query(r"select * from datapoints WHERE ChannelId = (?)")
        .bind(channel_id)
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

pub async fn add_data(
    State(app_state): State<AppState>,
    Json(payload): Json<AddData>,
) -> Result<impl axum::response::IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let data = models::Data {
        value: payload.value,
        timestamp: payload.timestamp.unwrap_or(chrono::Utc::now()),
        channel: payload.channel,
    };

    if app_state
        .channels
        .lock()
        .await
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
    });

    Ok((StatusCode::CREATED, Json(data)))
}

pub async fn add_channel(
    State(app_state): State<AppState>,
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
            });

        //we need to get a lock on the mutex and update channels, so that we keep the right list of channels in memory
        let mut channels = app_state.channels.lock().await;
        *channels = update_channels(&app_state.db).await.unwrap();

        //clone channels so we can read it
        return Ok((StatusCode::CREATED, Json(channels.clone())));
    } else {
        let error_response = serde_json::json!({
            "status": "error",
            "message": format!("Channel name cannot be empty"),
        });
        Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
    }
}

pub async fn get_channels(
    State(app_state): State<AppState>,
) -> Result<impl axum::response::IntoResponse, (StatusCode, Json<serde_json::Value>)> {
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
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AddData {
    pub value: f64,
    pub timestamp: Option<chrono::DateTime<Utc>>,
    pub channel: i64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AddChannel {
    pub channel_name: String,
}
