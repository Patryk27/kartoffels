mod create_bot;
mod create_world;
mod get_world;
mod get_worlds;

use crate::AppState;
use axum::extract::DefaultBodyLimit;
use axum::routing::{get, post};
use axum::Router;
use std::sync::Arc;
use tokio::sync::RwLock;

pub fn router() -> Router<Arc<RwLock<AppState>>> {
    Router::new()
        .route("/worlds", get(get_worlds::handle))
        .route("/worlds", post(create_world::handle))
        .route("/worlds/:world", get(get_world::handle))
        .route("/worlds/:world/bots", post(create_bot::handle))
        .layer(DefaultBodyLimit::max(512 * 1024))
}
