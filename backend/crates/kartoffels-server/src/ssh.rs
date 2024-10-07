mod channel;
mod client;
mod server;

use self::channel::*;
use self::client::*;
use self::server::*;
use anyhow::{anyhow, Context, Result};
use ed25519_dalek::SigningKey;
use itertools::Either;
use kartoffels_store::Store;
use rand::rngs::OsRng;
use russh::server::{Config, Server as _};
use russh_keys::key::KeyPair;
use std::pin::pin;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::{fs, select, time};
use tokio_util::sync::CancellationToken;
use tracing::info;

pub async fn start(
    socket: TcpListener,
    store: Arc<Store>,
    shutdown: CancellationToken,
) -> Result<()> {
    info!(addr = ?socket.local_addr()?, "starting ssh server");

    let key = load_key(&store).await?;

    let config = Arc::new(Config {
        inactivity_timeout: Some(Duration::from_secs(3600)),
        auth_rejection_time: Duration::from_secs(3),
        auth_rejection_time_initial: Some(Duration::from_secs(0)),
        keys: vec![key],
        ..Default::default()
    });

    info!("ready");

    let mut server = AppServer::new(store, shutdown.clone());
    let server = server.run_on_socket(config, &socket);
    let shutdown = shutdown.cancelled();

    let result = {
        let mut server = pin!(server);
        let mut shutdown = pin!(shutdown);

        select! {
            result = &mut server => Either::Left(result),
            _ = &mut shutdown => Either::Right(()),
        }
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

async fn load_key(store: &Store) -> Result<KeyPair> {
    let path = store.dir.join("ssh.key");

    if !path.exists() {
        info!(?path, "generating server key");

        let key = SigningKey::generate(&mut OsRng {}).to_bytes();

        fs::write(&path, key).await.with_context(|| {
            format!("couldn't write server key to `{}`", path.display())
        })?;
    }

    info!(?path, "loading server key");

    let key = fs::read(&path).await.with_context(|| {
        format!("couldn't load server key from `{}`", path.display())
    })?;

    let key = key.try_into().map_err(|_| {
        anyhow!("invalid server key found in `{}`", path.display())
    })?;

    let key = SigningKey::from_bytes(&key);

    Ok(KeyPair::Ed25519(key))
}
