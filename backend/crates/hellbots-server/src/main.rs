#![feature(let_chains)]
#![feature(try_blocks)]

mod endpoints;

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use hellbots::{World, WorldHandle, WorldId};
use std::collections::HashMap;
use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs::{self, File};
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tower_http::cors::{self, CorsLayer};
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing::{debug, info, warn};

#[derive(Debug, Parser)]
struct AppArgs {
    #[clap(long, default_value = "127.0.0.1:1313")]
    listen: SocketAddr,

    #[clap(long)]
    store: Option<PathBuf>,

    #[clap(long)]
    debug: bool,
}

#[derive(Debug)]
pub(crate) struct AppState {
    store: Option<PathBuf>,
    worlds: HashMap<WorldId, WorldHandle>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = AppArgs::parse();

    let filter = env::var("RUST_LOG").unwrap_or_else(|_| {
        let filter = if args.debug {
            "tower_http=debug,hellbots=debug"
        } else {
            "hellbots=info"
        };

        filter.to_owned()
    });

    tracing_subscriber::fmt().with_env_filter(filter).init();

    // ---

    info!(?args, "initializing");

    let state = init(args.store).await?;
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

async fn init(store: Option<PathBuf>) -> Result<AppState> {
    let mut worlds = HashMap::new();

    if let Some(store) = &store {
        debug!("checking store");

        let store_meta = fs::metadata(store).await.with_context(|| {
            format!("couldn't open store: {}", store.display())
        })?;

        if !store_meta.is_dir() {
            return Err(anyhow!(
                "store is not a directory: {}",
                store.display()
            ));
        }

        let mut entries = fs::read_dir(store).await?;

        while let Some(entry) = entries.next_entry().await? {
            let entry_path = entry.path();

            let Some(entry_stem) =
                entry_path.file_stem().and_then(|stem| stem.to_str())
            else {
                continue;
            };

            if entry.path().extension().and_then(|ext| ext.to_str())
                != Some("world")
            {
                continue;
            }

            info!("loading: {}", entry_path.display());

            let result: Result<()> = try {
                let id = entry_stem
                    .parse()
                    .context("couldn't extract world id from path")?;

                let world = File::options()
                    .read(true)
                    .write(true)
                    .open(&entry_path)
                    .await
                    .context("couldn't open world's file")?
                    .into_std()
                    .await;

                let world = World::resume(id, world)
                    .context("couldn't resume the world")?;

                worlds.insert(id, world);
            };

            result.with_context(|| {
                format!("couldn't load: {}", entry_path.display())
            })?;
        }
    } else {
        warn!("running without any store");
        warn!("all worlds will be deleted upon server's restart");
    }

    Ok(AppState { store, worlds })
}
