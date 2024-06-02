mod create_bot;
mod create_world;
mod get_world;
mod get_worlds;
mod update_bot;

use crate::AppState;
use axum::routing::{get, post, put};
use axum::Router;
use std::sync::Arc;
use tokio::sync::RwLock;

pub fn router() -> Router<Arc<RwLock<AppState>>> {
    Router::new()
        .route("/worlds", get(get_worlds::handle))
        .route("/worlds", post(create_world::handle))
        .route("/worlds/:world", get(get_world::handle))
        .route("/worlds/:world/bots", post(create_bot::handle))
        .route("/worlds/:world/bots/:bot", put(update_bot::handle))
}
