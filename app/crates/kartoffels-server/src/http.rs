mod endpoints;

use anyhow::Result;
use axum::extract::DefaultBodyLimit;
use axum::routing::{get, post};
use axum::Router;
use kartoffels_store::Store;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;
use tower_http::cors::{self, CorsLayer};
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing::info;

pub async fn start(
    socket: TcpListener,
    store: Arc<Store>,
    shutdown: CancellationToken,
) -> Result<()> {
    info!(addr = ?socket.local_addr()?, "starting");

    let app = {
        let cors = CorsLayer::new()
            .allow_methods(cors::Any)
            .allow_headers(cors::Any)
            .allow_origin(cors::Any);

        let trace = TraceLayer::new_for_http()
            .make_span_with(DefaultMakeSpan::default());

        let limit = DefaultBodyLimit::max(512 * 1024);

        Router::new()
            .route("/", get(endpoints::connect::handle))
            .route(
                "/sessions/:id/bots",
                post(endpoints::create_session_bot::handle),
            )
            .with_state((store, shutdown.clone()))
            .layer(cors)
            .layer(trace)
            .layer(limit)
    };

    info!("ready");

    let app = app.into_make_service_with_connect_info::<SocketAddr>();

    axum::serve(socket, app)
        .with_graceful_shutdown(shutdown.cancelled_owned())
        .await?;

    Ok(())
}
