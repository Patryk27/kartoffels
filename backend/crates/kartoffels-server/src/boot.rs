use crate::{AliveAppState, AppState};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::{fs, select, signal};
use tracing::{debug, info, warn};

pub async fn init(data: Option<PathBuf>) -> Result<Arc<RwLock<AppState>>> {
    let mut worlds = HashMap::new();

    if let Some(data) = &data {
        debug!(path = ?data, "checking data directory");

        let mut entries = fs::read_dir(data).await.with_context(|| {
            format!("couldn't open data directory: {}", data.display())
        })?;

        while let Some(entry) = entries.next_entry().await? {
            let entry_path = entry.path();

            let Some(entry_stem) =
                entry_path.file_stem().and_then(|stem| stem.to_str())
            else {
                continue;
            };

            let Some(entry_ext) =
                entry_path.extension().and_then(|ext| ext.to_str())
            else {
                continue;
            };

            match entry_ext {
                "new" => {
                    // TODO bail?

                    warn!(
                        path = ?entry_path,
                        "found a suspicious file (leftover from previous run)",
                    );
                }

                "world" => {
                    info!("loading: {}", entry_path.display());

                    let result: Result<()> = try {
                        let id = entry_stem
                            .parse()
                            .context("couldn't extract world id from path")?;

                        let world = kartoffels::resume(id, &entry_path)?;

                        worlds.insert(id, world);
                    };

                    result.with_context(|| {
                        format!(
                            "couldn't resume world: {}",
                            entry_path.display()
                        )
                    })?;
                }

                _ => {
                    // TODO bail?

                    warn!(
                        path = ?entry_path,
                        "found a suspicious file (not created by kartoffels)",
                    );
                }
            }
        }
    } else {
        warn!("running without any data directory");
    }

    Ok(Arc::new(RwLock::new(AppState::Alive(AliveAppState {
        data,
        worlds,
    }))))
}

pub async fn setup_shutdown_signal(state: Arc<RwLock<AppState>>) {
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

    let Some(state) = state.write().await.take() else {
        return;
    };

    for (_, world) in state.worlds {
        _ = world.close().await;
    }
}
