use anyhow::{Context, Result};
use futures_util::FutureExt;
use kartoffels_front::Frame;
use kartoffels_store::Store;
use rand::rngs::OsRng;
use russh::keys::ssh_key::private::{Ed25519PrivateKey, KeypairData};
use russh::keys::{Algorithm, PrivateKey};
use std::panic::AssertUnwindSafe;
use std::path::Path;
use std::pin::pin;
use std::sync::Arc;
use tokio::{fs, select};
use tokio_util::sync::CancellationToken;
use tracing::{info, instrument};

pub async fn start_session(
    store: Arc<Store>,
    mut frame: Frame,
    shutdown: CancellationToken,
) -> Result<()> {
    _ = frame.init().await;

    let sess = store.create_session().await?;

    let result = {
        let sess = kartoffels_front::main(&store, &sess, &mut frame);
        let sess = AssertUnwindSafe(sess).catch_unwind();
        let sess = pin!(sess);

        select! {
            result = sess => Some(result),
            _ = shutdown.cancelled() => None,
        }
    };

    _ = frame.destroy().await;

    match result {
        Some(Ok(result)) => {
            info!(?result, "session finished");
        }

        Some(Err(err)) => {
            if let Some(err) = err.downcast_ref::<String>() {
                info!(?err, "session crashed");
            } else {
                info!("session crashed");
            }

            _ = frame.send("ouch, the game has crashed\r\n".into()).await;
        }

        None => {
            info!("session aborted: server is shutting down");

            _ = frame
                .send("ouch, the server is shutting down\r\n".into())
                .await;
        }
    }

    Ok(())
}

#[instrument]
pub async fn load_key(path: &Path) -> Result<PrivateKey> {
    if !path.exists() {
        info!("generating server key");

        let key = PrivateKey::random(&mut OsRng, Algorithm::Ed25519)
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
