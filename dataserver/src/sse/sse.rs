use futures::stream::Stream;
use tokio_stream::wrappers::{errors::BroadcastStreamRecvError, BroadcastStream};

use axum::{
    extract::State,
    response::sse::{Event as SseEvent, KeepAlive, Sse},
};

use crate::controller::AppState;

pub async fn sse_handler(
    State(app_state): State<AppState>, //took at an Arc ->s (State<Arc<AppState>>)
) -> Sse<impl Stream<Item = Result<SseEvent, BroadcastStreamRecvError>>> {
    let rx = app_state.tx.subscribe();
    let mystream = BroadcastStream::new(rx);

    Sse::new(mystream).keep_alive(KeepAlive::default().text("keep-alive"))
}
