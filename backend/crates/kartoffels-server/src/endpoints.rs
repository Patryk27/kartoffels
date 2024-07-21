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
use tower_http::validate_request::ValidateRequestHeaderLayer;

pub fn router(
    state: Arc<RwLock<AppState>>,
    secret: Option<String>,
) -> Router<()> {
    let public_routes = Router::new()
        .route("/worlds", get(get_worlds::handle))
        .route("/worlds/:world", get(get_world::handle))
        .route("/worlds/:world/bots", post(create_bot::handle))
        .route("/worlds/:world/bots/:bot", get(get_world::handle_with_bot));

    let mut admin_routes =
        Router::new().route("/worlds", post(create_world::handle));

    if let Some(secret) = secret {
        admin_routes =
            admin_routes.layer(ValidateRequestHeaderLayer::bearer(&secret));
    }

    public_routes
        .merge(admin_routes)
        .layer(DefaultBodyLimit::max(512 * 1024))
        .with_state(state)
}
