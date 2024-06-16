use crate::{BotId, Map};
use glam::IVec2;
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::sync::Arc;

#[derive(Debug, Serialize)]
pub struct WorldUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<Arc<Value>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub map: Option<Arc<Map>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub bots: Option<Arc<BTreeMap<BotId, AnyBotUpdate>>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub bot: Option<BotUpdate>,
}

#[derive(Debug, Serialize)]
pub struct BotUpdate {
    pub uart: String,
}

#[derive(Debug, Serialize)]
pub struct AnyBotUpdate {
    pub pos: IVec2,
}
