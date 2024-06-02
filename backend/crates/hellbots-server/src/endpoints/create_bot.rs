use crate::AppState;
use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::Json;
use hellbots::{BotId, WorldId};
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::RwLock;

// TODO size limit
pub async fn handle(
    State(state): State<Arc<RwLock<AppState>>>,
    Path(world_id): Path<WorldId>,
    body: Bytes,
) -> impl IntoResponse {
    let world = state.read().await.worlds.get(&world_id).cloned().unwrap();
    let id = world.create_bot(body.to_vec()).await.unwrap();

    Json(Response { id })
}

#[derive(Debug, Serialize)]
struct Response {
    id: BotId,
}
