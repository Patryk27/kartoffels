use crate::error::AppResult;
use crate::AppState;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;
use kartoffels::prelude::{WorldId, WorldName};
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::RwLock;

pub async fn handle(
    State(state): State<Arc<RwLock<AppState>>>,
) -> AppResult<impl IntoResponse> {
    let state = state.read().await;

    let worlds = state
        .as_alive()?
        .worlds
        .iter()
        .map(|(id, world)| ResponseWorld {
            id: *id,
            name: world.name(),
        })
        .collect();

    Ok(Json(Response { worlds }).into_response())
}

#[derive(Debug, Serialize)]
struct Response<'a> {
    worlds: Vec<ResponseWorld<'a>>,
}

#[derive(Debug, Serialize)]
struct ResponseWorld<'a> {
    id: WorldId,
    name: &'a WorldName,
}
