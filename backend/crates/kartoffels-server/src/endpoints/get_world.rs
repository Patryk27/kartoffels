use crate::error::{AppError, AppResult};
use crate::AppState;
use anyhow::{Context, Result};
use axum::extract::ws::Message;
use axum::extract::{Path, State, WebSocketUpgrade};
use axum::response::IntoResponse;
use futures_util::{StreamExt, TryStreamExt};
use kartoffels::iface::{BotId, WorldId};
use std::sync::Arc;
use tokio::select;
use tokio::sync::RwLock;
use tracing::debug;

pub async fn handle1(
    State(state): State<Arc<RwLock<AppState>>>,
    Path(world_id): Path<WorldId>,
    socket: WebSocketUpgrade,
) -> AppResult<impl IntoResponse> {
    handle(state, world_id, None, socket).await
}

pub async fn handle2(
    State(state): State<Arc<RwLock<AppState>>>,
    Path((world_id, bot_id)): Path<(WorldId, BotId)>,
    socket: WebSocketUpgrade,
) -> AppResult<impl IntoResponse> {
    handle(state, world_id, Some(bot_id), socket).await
}

async fn handle(
    state: Arc<RwLock<AppState>>,
    world_id: WorldId,
    bot_id: Option<BotId>,
    socket: WebSocketUpgrade,
) -> AppResult<impl IntoResponse> {
    let mut updates = state
        .read()
        .await
        .world(world_id)?
        .join(bot_id)
        .await
        .map_err(AppError::MAP_HTTP_400)?;

    Ok(socket.on_upgrade(|mut socket| async move {
        debug!("socket opened");

        let result: Result<()> = try {
            loop {
                let update = select! {
                    update = updates.next() => {
                        update
                    }

                    Err(err) = socket.try_next() => {
                        break Err(err)?;
                    }
                };

                let msg = serde_json::to_string(&update)
                    .context("couldn't serialize message")?;

                socket
                    .send(Message::Text(msg))
                    .await
                    .context("couldn't send message")?;
            }
        };

        match result {
            Ok(_) => {
                debug!("socket closed");
            }
            Err(err) => {
                debug!("socket closed: {err:?}");
            }
        }
    }))
}
