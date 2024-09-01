mod http;
mod ssh;

use anyhow::{Context, Result};
use clap::Parser;
use indoc::indoc;
use kartoffels_store::Store;
use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
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
pub struct ServeCmd {
    data: PathBuf,

    #[clap(long)]
    http: Option<SocketAddr>,

    #[clap(long)]
    ssh: Option<SocketAddr>,

    #[clap(long)]
    debug: bool,

    #[clap(long)]
    quiet: bool,
}

impl ServeCmd {
    pub fn run(self) -> Result<()> {
        let filter = env::var("RUST_LOG").unwrap_or_else(|_| {
            let filter = if self.debug {
                "tower_http=debug,kartoffels=debug"
            } else {
                "kartoffels=info"
            };

            filter.to_owned()
        });

        if self.quiet {
            println!("starting");
        } else {
            tracing_subscriber::fmt()
                .with_env_filter(filter)
                .event_format(fmt::format::Format::default().without_time())
                .init();
        }

        for line in LOGO.lines() {
            info!("{}", line);
        }

        info!("");
        info!(?self, "starting");

        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?
            .block_on(async move {
                let store =
                    Store::open(&self.data).await.with_context(|| {
                        format!(
                            "couldn't load store from `{}`",
                            self.data.display()
                        )
                    })?;

                let store = Arc::new(store);
                let shutdown = CancellationToken::new();

                let http = {
                    let store = store.clone();
                    let shutdown = shutdown.clone();

                    async {
                        if let Some(addr) = &self.http {
                            http::start(addr, store, shutdown).await
                        } else {
                            Ok(())
                        }
                    }
                };

                let ssh = {
                    let store = store.clone();
                    let shutdown = shutdown.clone();

                    async {
                        if let Some(addr) = &self.ssh {
                            ssh::start(addr, store, shutdown).await
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

                info!("ready");

                try_join!(http, ssh, shutdown)?;

                Ok(())
            })
    }
}

async fn wait_for_shutdown() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");

        info!("ctrl-c signal detected, shutting down");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;

        info!("terminate signal detected, shutting down");
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
