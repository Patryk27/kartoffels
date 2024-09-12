use crate::common;
use anyhow::{Context, Result};
use axum::extract::ws::{Message, WebSocket};
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
use serde::Deserialize;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio::task;
use tokio_util::sync::CancellationToken;
use tower_http::cors::{self, CorsLayer};
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing::{info, info_span, Instrument};

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
    let span = info_span!("http", %addr);

    // We already buffer our stdout by relying on Ratatui, there's no need for
    // extra buffering on the socket's side
    let socket = socket.write_buffer_size(0);

    // We need ~256 Kb for *.elf file upload (128 Kb, but base64'd), let's round
    // to 512 Kb for good measure
    let socket = socket.max_message_size(512 * 1024);

    socket.on_upgrade(move |socket| {
        async move {
            info!("connection opened");

            match handle_connection(store, shutdown, socket).await {
                Ok(()) => {
                    info!("connection closed");
                }

                Err(err) => {
                    info!("connection closed: {:?}", err);
                }
            }
        }
        .instrument(span)
    })
}

async fn handle_connection(
    store: Arc<Store>,
    shutdown: CancellationToken,
    mut socket: WebSocket,
) -> Result<()> {
    let hello = recv_hello_msg(&mut socket)
        .await
        .context("couldn't retrieve hello message")?;

    let term =
        create_term(socket, hello).context("couldn't create terminal")?;

    common::start_session(term, store, shutdown).await;

    Ok(())
}

async fn recv_hello_msg(socket: &mut WebSocket) -> Result<HelloMsg> {
    let msg = socket
        .recv()
        .await
        .context("client disconnected")??
        .into_text()?;

    serde_json::from_str(&msg).context("couldn't deserialize message")
}

fn create_term(socket: WebSocket, hello: HelloMsg) -> Result<Term> {
    let (mut stdout, mut stdin) = socket.split();

    let stdin = {
        let (tx, rx) = mpsc::channel(1);

        task::spawn(
            async move {
                while let Some(msg) = stdin.next().await {
                    match msg {
                        Ok(Message::Text(msg)) => {
                            if tx.send(msg.into_bytes()).await.is_err() {
                                break;
                            }
                        }

                        Ok(Message::Binary(msg)) => {
                            if tx.send(msg).await.is_err() {
                                break;
                            }
                        }

                        Ok(_) => {
                            //
                        }

                        Err(_) => {
                            info!(
                                "couldn't pull data from socket, killing stdin"
                            );

                            return;
                        }
                    }
                }
            }
            .in_current_span(),
        );

        rx
    };

    let stdout = {
        let (tx, mut rx) = mpsc::channel::<Vec<u8>>(1);

        task::spawn(
            async move {
                while let Some(msg) = rx.recv().await {
                    if stdout.send(Message::Binary(msg)).await.is_err() {
                        info!("couldn't push data into socket, killing stdout");
                        return;
                    }
                }
            }
            .in_current_span(),
        );

        tx
    };

    let size = uvec2(hello.cols, hello.rows);
    let term = Term::new(TermType::Web, stdin, stdout, size)?;

    Ok(term)
}

#[derive(Clone, Debug, Deserialize)]
struct HelloMsg {
    cols: u32,
    rows: u32,
}
