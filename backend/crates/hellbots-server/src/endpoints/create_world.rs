use crate::error::{AppError, AppResult};
use crate::AppState;
use anyhow::Context;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use hellbots::{World, WorldConfig, WorldId};
use serde::Serialize;
use std::sync::Arc;
use tokio::fs::File;
use tokio::sync::RwLock;
use tracing::info;

// TODO must be password-protected
pub async fn handle(
    State(state): State<Arc<RwLock<AppState>>>,
    config: Json<WorldConfig>,
) -> AppResult<impl IntoResponse> {
    let mut state = state.write().await;
    let id = WorldId::new(&mut rand::thread_rng());
    let config = config.0;

    info!(?id, ?config, "creating new world");

    if state.has_world_named(&config.name) {
        return Err(AppError::Other(
            StatusCode::BAD_REQUEST,
            "world with this name already exists".into(),
        ));
    }

    let file = if let Some(data) = &state.data {
        let file = data.join(id.to_string()).with_extension("world");

        let file = File::create(&file)
            .await
            .context("couldn't create world's file")
            .map_err(AppError::MK_INTERNAL_SERVER_ERROR)?;

        Some(file.into_std().await)
    } else {
        None
    };

    let world =
        World::create(id, config, file).map_err(AppError::MK_BAD_REQUEST)?;

    state.worlds.insert(id, world);

    Ok(Json(Response { id }))
}

#[derive(Clone, Debug, Serialize)]
struct Response {
    id: WorldId,
}
