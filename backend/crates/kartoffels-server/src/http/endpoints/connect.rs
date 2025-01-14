use crate::common;
use anyhow::{Context, Result};
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{ConnectInfo, State, WebSocketUpgrade};
use axum::response::IntoResponse;
use flate2::write::GzEncoder;
use flate2::Compression;
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use glam::uvec2;
use kartoffels_store::Store;
use kartoffels_ui::{Stdin, Stdout, Term, TermFrontend};
use serde::Deserialize;
use std::io::Write;
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
    let (stdout, stdin) = socket.split();
    let stdin = create_term_stdin(stdin);
    let stdout = create_term_stdout(stdout);
    let size = uvec2(hello.cols, hello.rows);
    let term = Term::new(TermFrontend::Web, stdin, stdout, size)?;

    Ok(term)
}

fn create_term_stdin(mut stdin: SplitStream<WebSocket>) -> Stdin {
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
                        info!("couldn't pull data from socket, killing stdin");
                        return;
                    }
                }
            }
        }
        .in_current_span(),
    );

    rx
}

fn create_term_stdout(mut stdout: SplitSink<WebSocket, Message>) -> Stdout {
    let (tx, mut rx) = mpsc::channel::<Vec<u8>>(1);

    task::spawn(
        async move {
            while let Some(frame) = rx.recv().await {
                let frame = compress(&frame);

                if stdout.send(Message::Binary(frame)).await.is_err() {
                    info!("couldn't push data into socket, killing stdout");
                    return;
                }
            }
        }
        .in_current_span(),
    );

    tx
}

fn compress(frame: &[u8]) -> Vec<u8> {
    // Note that while encoding is a blocking operation, since the dataset we're
    // operating on is pretty small, using a dedicated thread-pool for encoding
    // doesn't make much sense.
    //
    // Quick benchmark says an average call to `compress()` finishes in ~100µs,
    // which is good enough not to block the runtime.

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());

    encoder.write_all(frame).unwrap();
    encoder.finish().unwrap()
}

#[derive(Clone, Debug, Deserialize)]
struct HelloMsg {
    cols: u32,
    rows: u32,
}
