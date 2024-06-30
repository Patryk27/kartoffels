use crate::{BotId, Map};
use chrono::{DateTime, Utc};
use glam::IVec2;
use serde::Serialize;
use serde_json::Value;
use std::collections::{BTreeMap, VecDeque};
use std::sync::Arc;

#[derive(Debug, Serialize)]
pub struct WorldUpdate {
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
    pub age: f32,
}

#[derive(Debug, Serialize)]
#[serde(tag = "status")]
pub enum ConnectedBotUpdate {
    #[serde(rename = "alive")]
    Alive { age: f32, serial: VecDeque<u32> },

    #[serde(rename = "dead")]
    Dead {
        msg: Arc<String>,
        killed_at: DateTime<Utc>,
    },

    #[serde(rename = "queued")]
    Queued {
        queue_place: usize,
        queue_len: usize,
        requeued: bool,
        msg: Option<Arc<String>>,
    },
}
