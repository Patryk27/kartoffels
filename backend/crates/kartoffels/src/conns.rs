mod systems;

pub use self::systems::*;
use crate::{BotEvent, BotEventRx, BotId, Dir, Map};
use glam::IVec2;
use serde::Serialize;
use serde_json::Value;
use std::collections::{BTreeMap, VecDeque};
use std::sync::Arc;
use tokio::sync::mpsc;

pub type ConnectionUpdateTx = mpsc::Sender<ConnectionUpdate>;
pub type ConnectionUpdateRx = mpsc::Receiver<ConnectionUpdate>;

#[derive(Debug)]
pub struct Connection {
    pub tx: ConnectionUpdateTx,
    pub bot: Option<ConnectionBot>,
    pub is_fresh: bool,
}

#[derive(Debug)]
pub struct ConnectionBot {
    pub id: BotId,
    pub events: Option<ConnectionBotEvents>,
}

#[derive(Debug)]
pub struct ConnectionBotEvents {
    pub rx: BotEventRx,
    pub init: Vec<Arc<BotEvent>>,
}

// TODO consider serializing just once and keeping `Arc<String>`
#[derive(Clone, Debug, Serialize)]
pub struct ConnectionUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<Arc<Value>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub map: Option<Arc<Map>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub bots: Option<Arc<BTreeMap<BotId, ConnectionBotUpdate>>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub bot: Option<ConnectionJoinedBotUpdate>,
}

#[derive(Debug, Serialize)]
pub struct ConnectionBotUpdate {
    pub pos: IVec2,
    pub dir: Dir,
    pub age: f32,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "status")]
pub enum ConnectionJoinedBotUpdate {
    #[serde(rename = "queued")]
    Queued {
        place: usize,
        requeued: bool,
        events: Vec<Arc<BotEvent>>,
    },

    #[serde(rename = "alive")]
    Alive {
        age: f32,
        serial: VecDeque<u32>,
        events: Vec<Arc<BotEvent>>,
    },

    #[serde(rename = "dead")]
    Dead { events: Vec<Arc<BotEvent>> },
}

#[derive(Debug)]
pub struct CreateConnection {
    pub id: Option<BotId>,
    pub tx: ConnectionUpdateTx,
}
