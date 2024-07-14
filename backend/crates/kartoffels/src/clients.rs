mod systems;

pub use self::systems::*;
use crate::{BotEvent, BotEventRx, BotId, Dir, Map};
use glam::IVec2;
use serde::Serialize;
use serde_json::Value;
use std::collections::{BTreeMap, VecDeque};
use std::sync::Arc;
use tokio::sync::mpsc;

pub type ClientUpdateTx = mpsc::Sender<ClientUpdate>;
pub type ClientUpdateRx = mpsc::Receiver<ClientUpdate>;

#[derive(Debug)]
pub struct Client {
    pub tx: ClientUpdateTx,
    pub bot: Option<ClientBot>,
    pub is_fresh: bool,
}

#[derive(Debug)]
pub struct ClientBot {
    pub id: BotId,
    pub events: Option<ClientBotEvents>,
}

#[derive(Debug)]
pub struct ClientBotEvents {
    pub rx: BotEventRx,
    pub init: Vec<Arc<BotEvent>>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ClientUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<Arc<Value>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub map: Option<Arc<Map>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub bots: Option<Arc<BTreeMap<BotId, ClientBotUpdate>>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub bot: Option<ClientConnectedBotUpdate>,
}

#[derive(Debug, Serialize)]
pub struct ClientBotUpdate {
    pub pos: IVec2,
    pub dir: Dir,
    pub age: f32,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "status")]
pub enum ClientConnectedBotUpdate {
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
pub struct CreateClient {
    pub id: Option<BotId>,
    pub tx: ClientUpdateTx,
}
