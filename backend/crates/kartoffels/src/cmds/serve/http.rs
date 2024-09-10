use anyhow::Result;
use axum::extract::ws::Message;
use axum::extract::{
    ConnectInfo, DefaultBodyLimit, State as AxumState, WebSocketUpgrade,
};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use futures_util::{SinkExt, StreamExt};
use glam::uvec2;
use kartoffels_store::Store;
use kartoffels_ui::{Term, TermType};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio::{select, task};
use tokio_util::sync::{CancellationToken, PollSender};
use tower_http::cors::{self, CorsLayer};
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing::info;

pub async fn start(
    addr: &SocketAddr,
    store: Arc<Store>,
    shutdown: CancellationToken,
) -> Result<()> {
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
            .with_state((store, shutdown.clone()))
            .layer(cors)
            .layer(trace)
            .layer(limit)
    };

    info!("ready");

    let app = app.into_make_service_with_connect_info::<SocketAddr>();

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown.cancelled_owned())
        .await?;

    Ok(())
}

async fn handle_connect(
    socket: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    AxumState((store, shutdown)): AxumState<(Arc<Store>, CancellationToken)>,
) -> impl IntoResponse {
    // We already buffer our stdout by relying on Ratatui, there's no need for
    // extra buffering on the socket's side
    let socket = socket.write_buffer_size(0);

    // We need ~256 Kb for *.elf file upload (128 Kb, but base64'd), let's round
    // to 512 Kb for good measure
    let socket = socket.max_message_size(512 * 1024);

    socket.on_upgrade(move |socket| async move {
        info!(?addr, "connection opened");

        let (mut stdout, stdin) = socket.split();

        let stdin = stdin.filter_map(|msg| async move {
            match msg {
                Ok(Message::Text(msg)) => Some(Ok(msg.into_bytes())),
                Ok(Message::Binary(msg)) => Some(Ok(msg)),
                Ok(_) => None,
                Err(err) => Some(Err(err.into())),
            }
        });

        let (stdout_tx, mut stdout_rx) = mpsc::channel(1);

        task::spawn(async move {
            while let Some(msg) = stdout_rx.recv().await {
                if let Err(err) = stdout.send(Message::Binary(msg)).await {
                    info!(?addr, "connection lost: {:?}", err);
                    break;
                }
            }
        });

        let stdout = PollSender::new(stdout_tx.clone())
            .with(|stdout| async move { Ok(stdout) });

        let mut term = {
            let stdin = Box::pin(stdin);
            let stdout = Box::pin(stdout);
            let size = uvec2(0, 0);

            Term::new(TermType::Web, stdin, stdout, size).await.unwrap()
        };

        let result = task::spawn(async move {
            kartoffels_game::main(&mut term, &store).await
        });

        let result = select! {
            result = result => Some(result),
            _ = shutdown.cancelled() => None,
        };

        match result {
            Some(Ok(result)) => {
                info!(?addr, "ui task finished: {:?}", result);
            }

            Some(Err(err)) => {
                info!(?addr, "ui task crashed: {}", err);

                _ = stdout_tx.send(Term::reset_cmds()).await;
                _ = stdout_tx.send(Term::crashed_msg()).await;
            }

            None => {
                info!(?addr, "ui task aborted: shutting down");

                _ = stdout_tx.send(Term::reset_cmds()).await;
                _ = stdout_tx.send(Term::shutting_down_msg()).await;
            }
        }

        info!(?addr, "connection closed");
    })
}
