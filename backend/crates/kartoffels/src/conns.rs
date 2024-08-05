mod systems;

pub use self::systems::*;
use crate::{BotEvent, BotEventRx, BotId, Dir, Map};
use glam::IVec2;
use serde::Serialize;
use serde_json::Value;
use std::collections::{BTreeMap, VecDeque};
use std::sync::Arc;
use tokio::sync::mpsc;

pub type ConnMsgTx = mpsc::Sender<ConnMsg>;
pub type ConnMsgRx = mpsc::Receiver<ConnMsg>;

#[derive(Debug)]
pub struct Conn {
    pub tx: ConnMsgTx,
    pub bot: Option<ConnBot>,
    pub is_fresh: bool,
}

#[derive(Debug)]
pub struct ConnBot {
    pub id: BotId,
    pub events: Option<ConnBotEvents>,
}

#[derive(Debug)]
pub struct ConnBotEvents {
    pub rx: BotEventRx,
    pub init: Vec<Arc<BotEvent>>,
}

// TODO consider serializing just once and keeping `Arc<String>`
#[derive(Clone, Debug, Serialize)]
pub struct ConnMsg {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<Arc<Value>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub map: Option<Arc<Map>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub bots: Option<Arc<BTreeMap<BotId, ConnBotUpdate>>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub bot: Option<ConnJoinedBotUpdate>,
}

#[derive(Debug, Serialize)]
pub struct ConnBotUpdate {
    pub pos: IVec2,
    pub dir: Dir,
    pub age: f32,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "status")]
pub enum ConnJoinedBotUpdate {
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

    #[serde(rename = "unknown")]
    Unknown,
}

#[derive(Debug)]
pub struct CreateConnection {
    pub id: Option<BotId>,
    pub tx: ConnMsgTx,
}
