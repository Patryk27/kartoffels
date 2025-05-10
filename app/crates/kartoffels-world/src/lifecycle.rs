mod systems;

pub use self::systems::*;
use crate::store::WorldBuffer;
use tokio::sync::oneshot;

#[derive(Debug)]
pub struct Shutdown {
    pub tx: Option<oneshot::Sender<WorldBuffer>>,
}
