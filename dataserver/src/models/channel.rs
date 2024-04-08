use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqliteRow, FromRow, Row};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Channel {
    pub channel_id: i64,
    pub channel_name: String,
}

// implements the cast from Sqliterow to Channel
// will need to write implementation for Postgres as well
impl<'r> FromRow<'r, SqliteRow> for Channel {
    fn from_row(row: &'r SqliteRow) -> Result<Self, sqlx::Error> {
        let channel_id = row.try_get("ChannelId")?;
        let channel_name = row.try_get("ChannelName")?;

        Ok(Channel {
            channel_id,
            channel_name,
        })
    }
}
