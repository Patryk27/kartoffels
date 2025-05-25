mod client;
mod cmds;
mod server;

use self::client::*;
use self::cmds::*;
use self::server::*;
use crate::common;
use anyhow::Result;
use itertools::Either;
use kartoffels_store::Store;
use rand::rngs::OsRng;
use russh::SshId;
use russh::keys::ssh_key::public::KeyData;
use russh::keys::{Algorithm, PrivateKey};
use russh::server::{Config, Server};
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::select;
use tokio_util::sync::CancellationToken;
use tracing::info;

type Admins = Option<HashSet<KeyData>>;

pub async fn start(
    admins: Admins,
    socket: TcpListener,
    store: Arc<Store>,
    shutdown: CancellationToken,
) -> Result<()> {
    info!(addr = ?socket.local_addr()?, "starting");

    let config = {
        let key = if store.testing() {
            PrivateKey::random(&mut OsRng, Algorithm::Ed25519)?
        } else {
            let key = store.dir().join("admin.key");

            common::load_key(&key).await?
        };

        Arc::new(Config {
            server_id: SshId::Standard(format!(
                "SSH-2.0-kartoffels_{}",
                env!("CARGO_PKG_VERSION")
            )),
            inactivity_timeout: Some(Duration::from_secs(60)),
            auth_rejection_time: Duration::from_secs(3),
            auth_rejection_time_initial: Some(Duration::from_secs(0)),
            keys: vec![key],
            nodelay: true,
            ..Default::default()
        })
    };

    info!("ready");

    let mut server = AppServer::new(admins, store);
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
            Ok(())
        }
    }
}
