use futures_util::FutureExt;
use kartoffels_store::Store;
use kartoffels_ui::Term;
use std::panic::AssertUnwindSafe;
use std::pin::pin;
use std::sync::Arc;
use tokio::select;
use tokio_util::sync::CancellationToken;
use tracing::info;

pub async fn start_session(
    mut term: Term,
    store: Arc<Store>,
    shutdown: CancellationToken,
) {
    _ = term.init().await;

    let result = {
        let session = kartoffels_session::main(&mut term, &store);
        let session = AssertUnwindSafe(session).catch_unwind();
        let session = pin!(session);

        select! {
            result = session => Some(result),
            _ = shutdown.cancelled() => None,
        }
    };

    _ = term.finalize().await;

    match result {
        Some(Ok(result)) => {
            info!("session finished: {:?}", result);
        }

        Some(Err(err)) => {
            if let Some(err) = err.downcast_ref::<String>() {
                info!("session crashed: {}", err);
            } else {
                info!("session crashed");
            }

            _ = term.send("ouch, the game has crashed\r\n".into()).await;
        }

        None => {
            info!("session aborted: server is shutting down");

            _ = term
                .send("ouch, the server is shutting down\r\n".into())
                .await;
        }
    }
}
