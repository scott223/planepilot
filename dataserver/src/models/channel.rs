use axum::response::sse::Event as SseEvent;
use serde::Serialize;
use sqlx::{sqlite::SqliteRow, FromRow, Row};
use tokio::sync::broadcast;

#[derive(Clone, Debug, Serialize)]
pub struct Channel {
    pub channel_id: i64,
    pub channel_name: String,
    #[serde(skip_serializing)]
    pub tx: broadcast::Sender<SseEvent>, //for the SSE broadcasts
}

// implements the cast from Sqliterow to Channel
// will need to write implementation for Postgres as well
impl<'r> FromRow<'r, SqliteRow> for Channel {
    fn from_row(row: &'r SqliteRow) -> Result<Self, sqlx::Error> {
        let channel_id = row.try_get("ChannelId")?;
        let channel_name = row.try_get("ChannelName")?;

        let (tx, _) = broadcast::channel::<SseEvent>(100);

        Ok(Channel {
            channel_id,
            channel_name,
            tx,
        })
    }
}
