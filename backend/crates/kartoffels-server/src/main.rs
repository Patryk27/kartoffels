#![feature(let_chains)]
#![feature(try_blocks)]

mod endpoints;
mod error;
mod state;

use crate::state::*;
use anyhow::{Context, Result};
use clap::Parser;
use indoc::indoc;
use kartoffels::World;
use std::collections::HashMap;
use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tower_http::cors::{self, CorsLayer};
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing::{debug, info, warn};

const LOGO: &str = indoc! {r#"
     _              _         __  __     _
    | |            | |       / _|/ _|   | |
    | | ____ _ _ __| |_ ___ | |_| |_ ___| |___
    | |/ / _` | '__| __/ _ \|  _|  _/ _ \ / __|
    |   < (_| | |  | || (_) | | | ||  __/ \__ \
    |_|\_\__,_|_|   \__\___/|_| |_| \___|_|___/
"#};

#[derive(Debug, Parser)]
struct AppArgs {
    #[clap(long, default_value = "127.0.0.1:1313")]
    listen: SocketAddr,

    #[clap(long)]
    data: Option<PathBuf>,

    #[clap(long)]
    debug: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = AppArgs::parse();

    let filter = env::var("RUST_LOG").unwrap_or_else(|_| {
        let filter = if args.debug {
            "tower_http=debug,kartoffels=debug"
        } else {
            "kartoffels=info"
        };

        filter.to_owned()
    });

    tracing_subscriber::fmt().with_env_filter(filter).init();

    // ---

    for line in LOGO.lines() {
        info!("{}", line);
    }

    info!("");
    info!(?args, "initializing");

    let state = init(args.data).await?;
    let state = Arc::new(RwLock::new(state));

    let listener = TcpListener::bind(&args.listen).await?;

    let app = {
        let cors = CorsLayer::new()
            .allow_methods(cors::Any)
            .allow_headers(cors::Any)
            .allow_origin(cors::Any);

        let trace = TraceLayer::new_for_http()
            .make_span_with(DefaultMakeSpan::default());

        endpoints::router()
            .layer(cors)
            .layer(trace)
            .with_state(state)
    };

    info!("ready");

    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

async fn init(data: Option<PathBuf>) -> Result<AppState> {
    let mut worlds = HashMap::new();

    if let Some(data) = &data {
        debug!(path = ?data, "checking data directory");

        let mut entries = fs::read_dir(data).await.with_context(|| {
            format!("couldn't open data directory: {}", data.display())
        })?;

        while let Some(entry) = entries.next_entry().await? {
            let entry_path = entry.path();

            let Some(entry_stem) =
                entry_path.file_stem().and_then(|stem| stem.to_str())
            else {
                continue;
            };

            let Some(entry_ext) =
                entry_path.extension().and_then(|ext| ext.to_str())
            else {
                continue;
            };

            match entry_ext {
                "new" => {
                    // TODO bail?

                    warn!(
                        path = ?entry_path,
                        "found a suspicious file (leftover from previous run)",
                    );
                }

                "world" => {
                    info!("loading: {}", entry_path.display());

                    let result: Result<()> = try {
                        let id = entry_stem
                            .parse()
                            .context("couldn't extract world id from path")?;

                        let world = World::resume(id, &entry_path)?;

                        worlds.insert(id, world);
                    };

                    result.with_context(|| {
                        format!(
                            "couldn't resume world: {}",
                            entry_path.display()
                        )
                    })?;
                }

                _ => {
                    // TODO bail?

                    warn!(
                        path = ?entry_path,
                        "found a suspicious file (not created by kartoffels)",
                    );
                }
            }
        }
    } else {
        warn!("running without any data directory");
    }

    Ok(AppState { data, worlds })
}
