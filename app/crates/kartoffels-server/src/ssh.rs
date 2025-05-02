mod channel;
mod client;
mod server;

use self::channel::*;
use self::client::*;
use self::server::*;
use anyhow::{Context, Result};
use itertools::Either;
use kartoffels_store::Store;
use rand::rngs::OsRng;
use russh::keys::ssh_key::private::{Ed25519PrivateKey, KeypairData};
use russh::keys::{Algorithm, PrivateKey};
use russh::server::{Config, Server as _};
use russh::{compression, Preferred};
use std::borrow::Cow;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::{fs, select, time};
use tokio_util::sync::CancellationToken;
use tracing::{info, instrument};

pub async fn start(
    socket: TcpListener,
    store: Arc<Store>,
    shutdown: CancellationToken,
) -> Result<()> {
    info!(addr = ?socket.local_addr()?, "starting ssh server");

    let key = load_key(&store.dir().join("ssh.key")).await?;

    let config = Arc::new(Config {
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
    });

    info!("ready");

    let mut server = AppServer::new(store, shutdown.clone());
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

            // TODO wait for clients to disconnect
            time::sleep(Duration::from_secs(1)).await;

            Ok(())
        }
    }
}

#[instrument]
async fn load_key(path: &Path) -> Result<PrivateKey> {
    if !path.exists() {
        info!("generating server key");

        let key = PrivateKey::random(&mut OsRng {}, Algorithm::Ed25519)
            .context("couldn't generate server key")?
            .to_bytes()
            .context("couldn't generate server key")?;

        fs::write(&path, key)
            .await
            .context("couldn't write server key")?;
    }

    info!("loading server key");

    let key = fs::read(&path).await.context("couldn't read server key")?;

    // TODO remove after kartoffels v0.9
    let key = if let Ok(key) = key.as_slice().try_into() {
        info!("converting server key to new format");

        let key = Ed25519PrivateKey::from_bytes(&key);

        let key = PrivateKey::new(KeypairData::Ed25519(key.into()), "")
            .context("couldn't convert server key")?;

        let key = key.to_bytes().context("couldn't convert server key")?;

        fs::write(&path, key)
            .await
            .context("couldn't write server key")?;

        fs::read(&path).await.context("couldn't read server key")?
    } else {
        key
    };

    PrivateKey::from_bytes(&key).context("couldn't parse server key")
}
