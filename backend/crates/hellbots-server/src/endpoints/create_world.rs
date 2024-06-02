use crate::AppState;
use axum::extract::State;
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
) -> impl IntoResponse {
    let mut state = state.write().await;
    let id = WorldId::new(&mut rand::thread_rng());
    let config = config.0;

    info!(?config, "creating new world");

    let store = if let Some(store) = &state.store {
        let store = store.join(id.to_string()).with_extension("world");
        let store = File::create(&store).await.unwrap(); // TODO unwrap

        Some(store.into_std().await)
    } else {
        None
    };

    let world = World::create(id, config, store).unwrap(); // TODO unwrap

    // TODO avoid duplicates
    state.worlds.insert(id, world);

    Json(Response { id })
}

#[derive(Clone, Debug, Serialize)]
struct Response {
    id: WorldId,
}
