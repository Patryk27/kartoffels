#![feature(map_try_insert)]

mod common;
pub mod http;
pub mod ssh;

use anyhow::{Context, Result};
use clap::Parser;
use indoc::indoc;
use kartoffels_store::{Secret, Store};
use kartoffels_world::prelude::Clock;
use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::{select, signal, try_join};
use tokio_util::sync::CancellationToken;
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
pub struct Cmd {
    store: PathBuf,

    #[clap(long)]
    http: Option<SocketAddr>,

    #[clap(long)]
    ssh: Option<SocketAddr>,

    #[clap(long)]
    secret: Option<Secret>,

    #[clap(long)]
    debug: bool,

    #[clap(long)]
    bench: bool,

    #[clap(long)]
    log_time: bool,
}

impl Cmd {
    pub fn run(self) -> Result<()> {
        self.init_tracing();
        self.print_logo();

        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?
            .block_on(self.start())
    }

    fn init_tracing(&self) {
        let filter = env::var("RUST_LOG").unwrap_or_else(|_| {
            if self.debug {
                "tower=debug,kartoffels=debug".into()
            } else {
                "kartoffels=info".into()
            }
        });

        if self.log_time {
            tracing_subscriber::fmt()
                .event_format(fmt::format::Format::default())
                .with_env_filter(filter)
                .init();
        } else {
            tracing_subscriber::fmt()
                .event_format(fmt::format::Format::default().without_time())
                .with_env_filter(filter)
                .init();
        }
    }

    fn print_logo(&self) {
        for line in LOGO.lines() {
            info!("{}", line);
        }

        info!("");
    }

    async fn start(self) -> Result<()> {
        info!("starting");

        let store = Store::new(Some(&self.store), self.secret)
            .await
            .with_context(|| {
                format!("couldn't open store at `{}`", self.store.display())
            })?;

        if self.bench {
            for world in store.public_worlds().iter() {
                world.overclock(Clock::Unlimited).await?;
            }
        }

        let store = Arc::new(store);
        let shutdown = CancellationToken::new();

        let http = {
            let store = store.clone();
            let shutdown = shutdown.clone();

            async {
                if let Some(addr) = self.http {
                    http::start(TcpListener::bind(addr).await?, store, shutdown)
                        .await
                } else {
                    Ok(())
                }
            }
        };

        let ssh = {
            let store = store.clone();
            let shutdown = shutdown.clone();

            async {
                if let Some(addr) = self.ssh {
                    ssh::start(TcpListener::bind(addr).await?, store, shutdown)
                        .await
                } else {
                    Ok(())
                }
            }
        };

        let shutdown = async {
            wait_for_shutdown().await;
            shutdown.cancel();
            store.close().await?;

            Ok(())
        };

        kartoffels_frontend::init();

        try_join!(http, ssh, shutdown)?;

        Ok(())
    }
}

async fn wait_for_shutdown() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");

        info!("got the C-c signal, shutting down");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;

        info!("got the termination signal, shutting down");
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
