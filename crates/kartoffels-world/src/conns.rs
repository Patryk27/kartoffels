mod systems;

pub use self::systems::*;
use crate::{BotId, Map};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::mpsc;

pub type ConnectionEventTx = mpsc::Sender<Arc<ConnState>>;
pub type ConnectionEventRx = mpsc::Receiver<Arc<ConnState>>;

#[derive(Debug)]
pub struct Conn {
    pub tx: ConnectionEventTx,
}

#[derive(Clone, Debug)]
pub struct ConnState {
}


#[derive(Debug)]
pub struct CreateConn {
    pub tx: ConnectionEventTx,
}
