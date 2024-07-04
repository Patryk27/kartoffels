use crate::{BotEvent, BotId, Dir, Map};
use glam::IVec2;
use serde::Serialize;
use serde_json::Value;
use std::collections::{BTreeMap, VecDeque};
use std::sync::Arc;
use tokio::sync::mpsc;

pub type UpdateTx = mpsc::Sender<Update>;
pub type UpdateRx = mpsc::Receiver<Update>;

#[derive(Clone, Debug, Serialize)]
pub struct Update {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<Arc<Value>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub map: Option<Arc<Map>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub bots: Option<Arc<BTreeMap<BotId, BotUpdate>>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub bot: Option<ConnectedBotUpdate>,
}

#[derive(Debug, Serialize)]
pub struct BotUpdate {
    pub pos: IVec2,
    pub dir: Dir,
    pub age: f32,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "status")]
pub enum ConnectedBotUpdate {
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
