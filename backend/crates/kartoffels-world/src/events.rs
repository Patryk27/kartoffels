mod stream;

pub use self::stream::*;
use crate::{BotId, ObjectId};
use glam::IVec2;
use tokio::sync::broadcast;

#[derive(Debug)]
pub struct Events {
    tx: Option<broadcast::Sender<EventLetter>>,
    pending: Vec<Event>,
}

impl Events {
    pub fn new(tx: Option<broadcast::Sender<EventLetter>>) -> Self {
        Self {
            tx,
            pending: Default::default(),
        }
    }

    pub fn add(&mut self, event: Event) {
        if self.tx.is_none() {
            return;
        }

        self.pending.push(event);
    }

    pub fn send(&mut self, version: u64) {
        let Some(tx) = &self.tx else {
            return;
        };

        for event in self.pending.drain(..) {
            _ = tx.send(EventLetter { event, version });
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Event {
    BotSpawned { id: BotId },
    BotKilled { id: BotId },
    BotMoved { id: BotId, at: IVec2 },
    ObjectPicked { id: ObjectId },
    ObjectDropped { id: ObjectId },
}

#[derive(Clone, Copy, Debug)]
pub struct EventLetter {
    pub event: Event,
    pub version: u64,
}
