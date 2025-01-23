use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use kartoffels_store::{SessionId, Store};
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

pub async fn handle(
    State((store, _)): State<(Arc<Store>, CancellationToken)>,
    Path(id): Path<SessionId>,
    body: Bytes,
) -> impl IntoResponse {
    let result =
        store.with_session(id, |sess| sess.complete_upload(body.to_vec()));

    match result {
        Some(Ok(())) => StatusCode::CREATED,
        Some(Err(())) => StatusCode::GONE,
        None => StatusCode::NOT_FOUND,
    }
}
