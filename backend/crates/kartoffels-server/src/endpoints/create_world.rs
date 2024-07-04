use crate::error::{AppError, AppResult};
use crate::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use kartoffels::{World, WorldConfig, WorldId};
use serde::Serialize;
use std::sync::Arc;
use tokio::runtime;
use tokio::sync::RwLock;
use tracing::info;

pub async fn handle(
    State(state): State<Arc<RwLock<AppState>>>,
    request: Json<WorldConfig>,
) -> AppResult<impl IntoResponse> {
    let mut state = state.write().await;
    let id = WorldId::new(&mut rand::thread_rng());
    let config = request.0;

    info!(?id, ?config, "creating new world");

    if state.has_world_named(&config.name) {
        return Err(AppError::Other(
            StatusCode::BAD_REQUEST,
            "world with this name already exists".into(),
        ));
    }

    let path = state
        .data
        .as_ref()
        .map(|data| data.join(id.to_string()).with_extension("world"));

    let world = World::create(runtime::Handle::current(), id, config, path)
        .map_err(AppError::MAP_HTTP_400)?;

    state.worlds.insert(id, world);

    Ok(Json(Response { id }))
}

#[derive(Clone, Debug, Serialize)]
struct Response {
    id: WorldId,
}
