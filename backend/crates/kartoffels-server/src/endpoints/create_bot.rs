use crate::error::{AppError, AppResult};
use crate::AppState;
use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::Json;
use kartoffels::prelude::{BotId, WorldId};
use serde::Serialize;
use std::borrow::Cow;
use std::sync::Arc;
use tokio::sync::RwLock;

pub async fn handle(
    State(state): State<Arc<RwLock<AppState>>>,
    Path(world_id): Path<WorldId>,
    body: Bytes,
) -> AppResult<impl IntoResponse> {
    let world = state.read().await.world(world_id)?;

    let id = world
        .upload_bot(Cow::Owned(body.to_vec()))
        .await
        .map_err(AppError::MAP_HTTP_400)?;

    Ok(Json(Response { id }))
}

#[derive(Debug, Serialize)]
struct Response {
    id: BotId,
}
