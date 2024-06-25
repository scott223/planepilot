use futures::stream::Stream;
use tokio_stream::wrappers::{errors::BroadcastStreamRecvError, BroadcastStream};

use axum::{
    extract::State,
    http::StatusCode,
    response::sse::{Event as SseEvent, KeepAlive, Sse},
    Json,
};

use crate::controller::AppState;

pub async fn sse_handler_general_channel(
    State(app_state): State<AppState>, //took at an Arc ->s (State<Arc<AppState>>)
) -> Result<
    Sse<impl Stream<Item = Result<SseEvent, BroadcastStreamRecvError>>>,
    (StatusCode, Json<serde_json::Value>),
> {
    let rx = app_state.tx.subscribe();
    let mystream = BroadcastStream::new(rx);

    Ok(Sse::new(mystream).keep_alive(KeepAlive::default().text("keep-alive")))
}
