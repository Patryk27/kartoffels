use anyhow::Result;
use axum::extract::{DefaultBodyLimit, State as AxumState, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use kartoffels_store::Store;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::{self, CorsLayer};
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing::info;

pub async fn start(addr: &SocketAddr, store: Arc<Store>) -> Result<()> {
    info!(?addr, "starting http server");

    let listener = TcpListener::bind(&addr).await?;

    let app = {
        let cors = CorsLayer::new()
            .allow_methods(cors::Any)
            .allow_headers(cors::Any)
            .allow_origin(cors::Any);

        let trace = TraceLayer::new_for_http()
            .make_span_with(DefaultMakeSpan::default());

        let limit = DefaultBodyLimit::max(512 * 1024);

        Router::new()
            .route("/", get(handle_connect))
            .with_state(store)
            .layer(cors)
            .layer(trace)
            .layer(limit)
    };

    info!("ready");

    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

async fn handle_connect(
    AxumState(_store): AxumState<Arc<Store>>,
    socket: WebSocketUpgrade,
) -> impl IntoResponse {
    socket.on_upgrade(|_socket| async move {
        // info!("connection opened");

        // let (stdout, stdin) = socket.split();

        // let stdin = stdin.map(|msg| match msg {
        //     Ok(Message::Binary(msg)) if msg.len() == 1 => Ok(msg[0]),
        //     Ok(msg) => Err(anyhow!(
        //         "unexpected message type: {:?}",
        //         mem::discriminant(&msg)
        //     )),
        //     Err(err) => Err(err.into()),
        // });

        // let stdout =
        //     stdout.with(|stdout| async move { Ok(Message::Binary(stdout)) });

        // let stdin = Stdin::new(stdin);
        // let stdout = Stdout::new(stdout);

        // match ui::start(&server, stdin, stdout).await {
        //     Ok(()) => {
        //         info!("connection closed");
        //     }
        //     Err(err) => {
        //         info!("connection closed: {:?}", err);
        //     }
        // }
    })
}
