mod stream;
mod systems;

pub use self::stream::*;
pub use self::systems::*;
use crate::{BotId, ObjectId};
use bevy_ecs::event::Event as BevyEvent;
use bevy_ecs::system::Resource;
use glam::IVec2;
use tokio::sync::broadcast;

#[derive(Debug, Resource)]
pub struct Events {
    pub tx: broadcast::Sender<EventLetter>,
    pub pending: Vec<Event>,
}

impl Events {
    pub fn send(&mut self, version: u64) {
        for event in self.pending.drain(..) {
            _ = self.tx.send(EventLetter { event, version });
        }
    }
}

#[derive(Clone, Copy, Debug, BevyEvent)]
pub enum Event {
    BotBorn { id: BotId },
    BotDied { id: BotId },
    BotMoved { id: BotId, at: IVec2 },
    BotScored { id: BotId },
    BotDiscarded { id: BotId },
    ObjectPicked { id: ObjectId },
    ObjectDropped { id: ObjectId },
}

#[derive(Clone, Copy, Debug)]
pub struct EventLetter {
    pub event: Event,
    pub version: u64,
}
