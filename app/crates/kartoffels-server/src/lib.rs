#![feature(map_try_insert)]
#![feature(result_flattening)]
#![feature(try_blocks)]

mod common;

pub mod admin;
pub mod http;
pub mod ssh;

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use indoc::indoc;
use kartoffels_store::Store;
use russh::keys::PublicKey;
use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::{select, signal, time, try_join};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};
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
    admin: Option<SocketAddr>,

    #[clap(long = "admin-key")]
    admin_keys: Vec<PublicKey>,

    #[clap(long)]
    open_admin: bool,

    #[clap(long)]
    http: Option<SocketAddr>,

    #[clap(long)]
    ssh: Option<SocketAddr>,

    #[clap(long)]
    debug: bool,

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

        if self.debug {
            warn!("--debug is active");
        }

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

        kartoffels_front::init();

        let store =
            Store::open(Some(&self.store), false)
                .await
                .with_context(|| {
                    format!("couldn't open store at `{}`", self.store.display())
                })?;

        let store = Arc::new(store);
        let shutdown = CancellationToken::new();

        let admin = self.admin(store.clone(), shutdown.clone());
        let http = self.http(store.clone(), shutdown.clone());
        let ssh = self.ssh(store.clone(), shutdown.clone());
        let watchdog = self.watchdog(&store, &shutdown);
        let terminate = self.terminate(&store, &shutdown);

        let result = try_join!(admin, http, ssh, watchdog, terminate);

        match result {
            Ok(_) => {
                info!("shut down, goodbye");

                Ok(())
            }

            Err(err) => {
                error!(?err, "whoopsie");
                error!("shutting down");

                if let Err(err) = store.close().await {
                    warn!(?err, "couldn't close store");
                }

                Err(err)
            }
        }
    }

    async fn admin(
        &self,
        store: Arc<Store>,
        shutdown: CancellationToken,
    ) -> Result<()> {
        let Some(addr) = self.admin else {
            return Ok(());
        };

        let keys = if self.open_admin {
            warn!(
                "--open-admin is active, everyone will have access to the \
                 admin panel",
            );

            None
        } else {
            let keys = self
                .admin_keys
                .iter()
                .map(|key| key.key_data().to_owned())
                .collect();

            Some(keys)
        };

        let socket = TcpListener::bind(addr).await?;

        admin::start(keys, socket, store, shutdown).await
    }

    async fn http(
        &self,
        store: Arc<Store>,
        shutdown: CancellationToken,
    ) -> Result<()> {
        let Some(addr) = self.http else {
            return Ok(());
        };

        let socket = TcpListener::bind(addr).await?;

        http::start(socket, store, shutdown).await
    }

    async fn ssh(
        &self,
        store: Arc<Store>,
        shutdown: CancellationToken,
    ) -> Result<()> {
        let Some(addr) = self.ssh else {
            return Ok(());
        };

        let socket = TcpListener::bind(addr).await?;

        ssh::start(socket, store, shutdown).await
    }

    async fn watchdog(
        &self,
        store: &Store,
        shutdown: &CancellationToken,
    ) -> Result<()> {
        let task = async {
            loop {
                time::timeout(Duration::from_secs(10), store.ping())
                    .await
                    .map_err(|_| anyhow!("timed out"))
                    .flatten()
                    .context("couldn't ping store")?;

                time::sleep(Duration::from_secs(10)).await;
            }
        };

        select! {
            result = task => result,
            _ = shutdown.cancelled() => Ok(())
        }
    }

    async fn terminate(
        &self,
        store: &Store,
        shutdown: &CancellationToken,
    ) -> Result<()> {
        let ctrl_c = async {
            signal::ctrl_c()
                .await
                .expect("failed to install C-c handler");

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
        };

        shutdown.cancel();
        store.close().await?;

        Ok(())
    }
}
