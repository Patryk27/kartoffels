use crate::common;
use anyhow::{Context, Result};
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{ConnectInfo, State, WebSocketUpgrade};
use axum::response::IntoResponse;
use futures_util::{SinkExt, StreamExt};
use glam::uvec2;
use kartoffels_store::Store;
use kartoffels_ui::{Term, TermType};
use serde::Deserialize;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::task;
use tokio_util::sync::CancellationToken;
use tracing::{info, info_span, Instrument};

pub async fn handle(
    socket: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State((store, shutdown)): State<(Arc<Store>, CancellationToken)>,
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

            match main(store, shutdown, socket).await {
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

async fn main(
    store: Arc<Store>,
    shutdown: CancellationToken,
    mut socket: WebSocket,
) -> Result<()> {
    let hello = recv_hello_msg(&mut socket)
        .await
        .context("couldn't retrieve hello message")?;

    let term =
        create_term(socket, hello).context("couldn't create terminal")?;

    common::start_session(store, term, shutdown).await;

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
