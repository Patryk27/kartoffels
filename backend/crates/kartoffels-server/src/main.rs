#![feature(let_chains)]
#![feature(try_blocks)]

mod boot;
mod endpoints;
mod error;
mod state;

use crate::state::*;
use anyhow::Result;
use clap::Parser;
use indoc::indoc;
use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;
use tokio::net::TcpListener;
use tower_http::cors::{self, CorsLayer};
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing::info;
use tracing_subscriber::fmt;

const LOGO: &str = indoc! {r#"
     _              _         __  __     _
    | |            | |       / _|/ _|   | |
    | | ____ _ _ __| |_ ___ | |_| |_ ___| |___
    | |/ / _` | '__| __/ _ \|  _|  _/ _ \ / __|
    |   < (_| | |  | || (_) | | | ||  __/ \__ \
    |_|\_\__,_|_|   \__\___/|_| |_| \___|_|___/
"#};

#[derive(Debug, Parser)]
struct Args {
    #[clap(long, default_value = "127.0.0.1:1313")]
    listen: SocketAddr,

    #[clap(long)]
    data: Option<PathBuf>,

    #[clap(long)]
    secret: Option<String>,

    #[clap(long)]
    debug: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let filter = env::var("RUST_LOG").unwrap_or_else(|_| {
        let filter = if args.debug {
            "tower_http=debug,kartoffels=debug"
        } else {
            "kartoffels=info"
        };

        filter.to_owned()
    });

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .event_format(fmt::format::Format::default().without_time())
        .init();

    // ---

    for line in LOGO.lines() {
        info!("{}", line);
    }

    info!("");
    info!(?args, "initializing");

    let state = boot::init(args.data).await?;
    let signal = boot::setup_shutdown_signal(state.clone());
    let listener = TcpListener::bind(&args.listen).await?;

    let app = {
        let cors = CorsLayer::new()
            .allow_methods(cors::Any)
            .allow_headers(cors::Any)
            .allow_origin(cors::Any);

        let trace = TraceLayer::new_for_http()
            .make_span_with(DefaultMakeSpan::default());

        endpoints::router(state, args.secret)
            .layer(cors)
            .layer(trace)
    };

    info!("ready");

    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(signal)
        .await?;

    Ok(())
}
