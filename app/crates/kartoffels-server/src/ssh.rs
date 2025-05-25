mod channel;
mod client;
mod server;

use self::channel::*;
use self::client::*;
use self::server::*;
use crate::common;
use anyhow::Result;
use itertools::Either;
use kartoffels_store::Store;
use russh::server::{Config, Server as _};
use russh::{Preferred, SshId, compression};
use std::borrow::Cow;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::{select, time};
use tokio_util::sync::CancellationToken;
use tracing::info;

pub async fn start(
    socket: TcpListener,
    store: Arc<Store>,
    shutdown: CancellationToken,
) -> Result<()> {
    info!(addr = ?socket.local_addr()?, "starting");

    let config = {
        let key = store.dir().join("ssh.key");
        let key = common::load_key(&key).await?;

        Arc::new(Config {
            server_id: SshId::Standard(format!(
                "SSH-2.0-kartoffels_{}",
                env!("CARGO_PKG_VERSION")
            )),
            inactivity_timeout: Some(Duration::from_secs(3600)),
            auth_rejection_time: Duration::from_secs(3),
            auth_rejection_time_initial: Some(Duration::from_secs(0)),
            preferred: Preferred {
                compression: Cow::Owned(vec![
                    compression::ZLIB,
                    compression::ZLIB_LEGACY,
                ]),
                ..Default::default()
            },
            keys: vec![key],
            nodelay: true,
            ..Default::default()
        })
    };

    info!("ready");

    let clients = Arc::new(AtomicUsize::new(0));

    let mut server = AppServer::new(store, shutdown.clone(), clients.clone());
    let server = server.run_on_socket(config, &socket);
    let shutdown = shutdown.cancelled();

    let result = select! {
        result = server => Either::Left(result),
        _ = shutdown => Either::Right(()),
    };

    match result {
        Either::Left(result) => Ok(result?),

        Either::Right(_) => {
            info!("shutting down");

            if clients.load(Ordering::SeqCst) > 0 {
                time::sleep(Duration::from_secs(1)).await;
            }

            Ok(())
        }
    }
}
