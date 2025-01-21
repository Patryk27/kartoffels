mod alive;
mod dead;
mod queued;
mod systems;

pub use self::alive::*;
pub use self::dead::*;
pub use self::queued::*;
pub use self::systems::*;
use crate::{AliveBot, BotId, CreateBotRequest, Dir, QueuedBot};
use anyhow::Result;
use bevy_ecs::event::Event;
use bevy_ecs::system::Resource;
use glam::IVec2;
use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;

#[derive(Clone, Debug, Default, Resource, Serialize, Deserialize)]
pub struct Bots {
    pub alive: AliveBots,
    pub dead: DeadBots,
    pub queued: QueuedBots,
}

impl Bots {
    pub fn contains(&self, id: BotId) -> bool {
        self.alive.contains(id)
            || self.dead.contains(id)
            || self.queued.contains(id)
    }

    pub fn remove(&mut self, id: BotId) {
        self.alive.remove(id);
        self.dead.remove(id);
        self.queued.remove(id);
    }
}

#[derive(Clone, Copy, Debug, Default, Resource)]
pub struct Spawn {
    pub pos: Option<IVec2>,
    pub dir: Option<Dir>,
}

#[derive(Debug, Event)]
pub struct CreateBot {
    pub req: Option<CreateBotRequest>,
    pub tx: Option<oneshot::Sender<Result<BotId>>>,
}

#[derive(Debug, Event)]
pub struct SpawnBot {
    pub bot: Option<Box<QueuedBot>>,
    pub tx: Option<oneshot::Sender<Result<BotId>>>,
    pub requeue_if_cant_spawn: bool,
}

#[derive(Debug, Event)]
pub struct KillBot {
    pub killed: Option<Box<AliveBot>>,
    pub reason: String,
    pub killer: Option<BotId>,
}
