use futures::stream::Stream;
use tokio_stream::wrappers::{errors::BroadcastStreamRecvError, BroadcastStream};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::sse::{Event as SseEvent, KeepAlive, Sse},
    Json,
};

use crate::{controller::AppState, models::Channel};

pub async fn sse_handler(
    State(app_state): State<AppState>, //took at an Arc ->s (State<Arc<AppState>>)
    Path(channel_id): Path<i64>,
) -> Result<
    Sse<impl Stream<Item = Result<SseEvent, BroadcastStreamRecvError>>>,
    (StatusCode, Json<serde_json::Value>),
> {
    let channels = app_state.channels.lock().await;
    let chnl: Vec<&Channel> = channels
        .iter()
        .filter(|c| c.channel_id == channel_id)
        .collect();

    if chnl.len() != 1 {
        let error_response = serde_json::json!({
            "status": "error",
            "message": format!("Channel with this id does not exist"),
        });

        return Err((StatusCode::BAD_REQUEST, Json(error_response)));
    };

    let rx = chnl[0].tx.subscribe();
    let mystream = BroadcastStream::new(rx);

    Ok(Sse::new(mystream).keep_alive(KeepAlive::default().text("keep-alive")))
}
