use crate::error::AppResult;
use crate::AppState;
use axum::extract::ws::Message;
use axum::extract::{Path, State, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum::Error;
use futures_util::{FutureExt, Sink, SinkExt, Stream, StreamExt};
use glam::{ivec2, IVec2};
use hellbots::{BotId, Tile, WorldHandle, WorldId};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time;
use tracing::debug;

pub async fn handle(
    State(state): State<Arc<RwLock<AppState>>>,
    Path(world_id): Path<WorldId>,
    socket: WebSocketUpgrade,
) -> AppResult<impl IntoResponse> {
    let world = state.read().await.world(world_id)?;

    Ok(socket.on_upgrade(|socket| async move {
        debug!("socket opened");

        let (tx, rx) = socket.split();

        Task::new(world).await.main(tx, rx).await;

        debug!("socket closed");
    }))
}

#[derive(Debug)]
struct Task {
    world: WorldHandle,
    camera_pos: IVec2,
    camera_size: IVec2,
    connected_to: Option<BotId>,
    following: bool,
}

impl Task {
    async fn new(world: WorldHandle) -> Self {
        let camera_pos = world.snapshot().read().await.center();

        Self {
            world,
            camera_pos,
            camera_size: ivec2(0, 0),
            connected_to: None,
            following: false,
        }
    }

    async fn main(
        mut self,
        mut tx: impl Sink<Message, Error = Error> + Unpin,
        mut rx: impl Stream<Item = AppResult<Message, Error>> + Unpin,
    ) {
        let mut penalty = None;

        loop {
            while let Some(Some(msg)) = rx.next().now_or_never() {
                self.process_msg(msg);
            }

            let msg = self.prepare_out_msg().await;

            if let Err(err) = tx.send(Message::Text(msg)).await {
                debug!("couldn't send message: {err:?}");
                break;
            }

            match time::timeout(Duration::from_millis(125), rx.next()).await {
                Ok(None) => {
                    // Client has disconnected
                    break;
                }

                Ok(Some(msg)) => {
                    if let Some(penalty) = penalty.take() {
                        // If client is spamming us with messages, throttle it
                        penalty.await;
                    }

                    self.process_msg(msg);

                    penalty = Some(time::sleep(Duration::from_millis(100)));
                }

                Err(_) => {
                    // We've timeouted, send newest state to the client
                    continue;
                }
            }
        }
    }

    fn process_msg(&mut self, msg: AppResult<Message, Error>) {
        let msg = match &msg {
            Ok(msg) => msg,
            Err(err) => {
                debug!("couldn't read message: {err:?}");
                return;
            }
        };

        let msg = match msg.to_text() {
            Ok(msg) => msg,
            Err(_) => {
                debug!("couldn't read message: not a string");
                return;
            }
        };

        let msg: Request = match serde_json::from_str(msg) {
            Ok(msg) => msg,
            Err(err) => {
                debug!("couldn't parse message: {err:?}");
                return;
            }
        };

        debug!(?msg, "processing message");

        match msg {
            Request::MoveCamera { dx, dy } => {
                self.camera_pos.x += dx.unwrap_or(0);
                self.camera_pos.y += dy.unwrap_or(0);
            }

            Request::ScaleCamera { x, dx, y, dy } => {
                if let Some(x) = x {
                    self.camera_size.x = x;
                } else {
                    self.camera_size.x += dx.unwrap_or(0);
                }

                if let Some(y) = y {
                    self.camera_size.y = y;
                } else {
                    self.camera_size.y += dy.unwrap_or(0);
                }
            }

            Request::ConnectToBot { id } => {
                self.connected_to = Some(id);
            }

            Request::DisconnectFromBot => {
                self.connected_to = None;
            }

            Request::FollowBot => {
                self.following = true;
            }

            Request::UnfollowBot => {
                self.following = false;
            }
        }
    }

    async fn prepare_out_msg(&mut self) -> String {
        let world = self.world.snapshot().read().await;
        let bot = self.connected_to.and_then(|id| world.bot(id));

        if self.following
            && let Some(bot) = bot
        {
            self.camera_pos = bot.pos;
        }

        let map = if self.camera_size.x > 0 && self.camera_size.y > 0 {
            let camera_min = self.camera_pos - self.camera_size / 2;
            let camera_max = self.camera_pos + (self.camera_size + 1) / 2;

            world.map(camera_min, camera_max)
        } else {
            Default::default()
        };

        let bot = if let Some(bot_id) = self.connected_to {
            if let Some(bot) = world.bot(bot_id) {
                Some(ResponseConnectedBot { uart: &bot.uart })
            } else {
                self.connected_to = None;

                None
            }
        } else {
            None
        };

        let bots = world.bots().map(|(id, _)| ResponseBot { id }).collect();

        serde_json::to_string(&Response {
            map,
            mode: world.mode(),
            bot,
            bots,
        })
        .unwrap()
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "op", content = "args", rename_all = "kebab-case")]
enum Request {
    MoveCamera {
        dx: Option<i32>,
        dy: Option<i32>,
    },

    ScaleCamera {
        x: Option<i32>,
        dx: Option<i32>,
        y: Option<i32>,
        dy: Option<i32>,
    },

    ConnectToBot {
        id: BotId,
    },

    DisconnectFromBot,

    FollowBot,
    UnfollowBot,
}

#[derive(Debug, Serialize)]
struct Response<'a> {
    mode: &'a Value,
    map: Vec<Tile>,
    bot: Option<ResponseConnectedBot<'a>>,
    bots: Vec<ResponseBot>,
}

#[derive(Debug, Serialize)]
struct ResponseConnectedBot<'a> {
    uart: &'a str,
}

#[derive(Debug, Serialize)]
struct ResponseBot {
    id: BotId,
}
