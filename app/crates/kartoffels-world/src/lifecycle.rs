mod systems;

pub use self::systems::*;
use crate::*;

#[derive(Debug)]
pub struct Shutdown {
    pub tx: Option<oneshot::Sender<WorldBuffer>>,
}
