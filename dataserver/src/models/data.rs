use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqliteRow, FromRow, Row};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Data {
    pub value: f64,
    pub timestamp: chrono::DateTime<Utc>,
    pub channel_name: String,
}

// implements the cast from Sqliterow to Data
// will need to write implementation for Postgres as well
impl<'r> FromRow<'r, SqliteRow> for Data {
    fn from_row(row: &'r SqliteRow) -> Result<Self, sqlx::Error> {
        let value = row.try_get("DataPointValue")?;
        let timestamp = row.try_get("CreationDate")?;
        let channel_name = row.try_get("ChannelName")?;

        Ok(Data {
            value,
            timestamp,
            channel_name,
        })
    }
}

//to pass as SSE payload
impl<'a> AsRef<Data> for Data {
    fn as_ref(&self) -> &Data {
        self
    }
}
