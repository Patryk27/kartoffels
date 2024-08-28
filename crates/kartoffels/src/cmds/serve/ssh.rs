mod channel;
mod client;
mod server;

use self::channel::*;
use self::client::*;
use self::server::*;
use anyhow::{anyhow, Context, Result};
use ed25519_dalek::SigningKey;
use kartoffels_store::Store;
use rand::rngs::OsRng;
use russh::server::{Config, Server as _};
use russh_keys::key::KeyPair;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::fs;
use tracing::info;

pub async fn start(addr: &SocketAddr, store: Arc<Store>) -> Result<()> {
    info!(?addr, "starting ssh server");

    let key = load_key(&store).await?;

    let config = Arc::new(Config {
        inactivity_timeout: Some(Duration::from_secs(3600)),
        auth_rejection_time: Duration::from_secs(3),
        auth_rejection_time_initial: Some(Duration::from_secs(0)),
        keys: vec![key],
        ..Default::default()
    });

    info!("ready");

    AppServer::new(store).run_on_address(config, addr).await?;

    Ok(())
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
