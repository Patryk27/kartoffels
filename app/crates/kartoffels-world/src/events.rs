mod stream;

pub use self::stream::*;
use crate::{BotId, ObjectId};
use glam::IVec2;
use tokio::sync::broadcast;

#[derive(Debug)]
pub struct Events {
    pub tx: Option<broadcast::Sender<EventEnvelope>>,
    pub pending: Vec<Event>,
}

impl Events {
    pub fn add(&mut self, event: Event) {
        if self.tx.is_none() {
            return;
        }

        self.pending.push(event);
    }

    pub fn send(&mut self, version: u64) {
        let Some(tx) = &mut self.tx else {
            return;
        };

        for event in self.pending.drain(..) {
            _ = tx.send(EventEnvelope { event, version });
        }
    }
}

#[derive(Clone, Copy, Debug)]
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
pub struct EventEnvelope {
    pub event: Event,
    pub version: u64,
}
