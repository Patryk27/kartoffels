use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Policy {
    pub max_alive_bots: usize,
    pub max_queued_bots: usize,
}
