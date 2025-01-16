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
    store: Arc<Store>,
    mut term: Term,
    shutdown: CancellationToken,
) {
    _ = term.init().await;

    let sess = store.create_session();

    let result = {
        let sess = kartoffels_session::main(&store, &sess, &mut term);
        let sess = AssertUnwindSafe(sess).catch_unwind();
        let sess = pin!(sess);

        select! {
            result = sess => Some(result),
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
