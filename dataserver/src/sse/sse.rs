use futures::stream::Stream;
use tokio_stream::wrappers::{errors::BroadcastStreamRecvError, BroadcastStream};

use crate::controller::AppState;

use axum::{
    extract::State,
    response::sse::{Event as SseEvent, KeepAlive, Sse},
};

pub async fn sse_handler(
    State(app_state): State<AppState>, //took at an Arc ->s (State<Arc<AppState>>)
) -> Sse<impl Stream<Item = Result<SseEvent, BroadcastStreamRecvError>>> {
    let rx = app_state.tx.subscribe();
    let mystream = BroadcastStream::new(rx);

    Sse::new(mystream).keep_alive(KeepAlive::default())
}
