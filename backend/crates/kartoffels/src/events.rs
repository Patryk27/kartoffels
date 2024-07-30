use crate::BotId;
use serde::Serialize;
use tokio::sync::mpsc;

pub type EventTx = mpsc::Sender<Event>;
pub type EventRx = mpsc::Receiver<Event>;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "ty")]
pub enum Event {
    #[serde(rename = "bot-killed")]
    BotKilled { id: BotId, killer: Option<BotId> },
}
