mod systems;

pub use self::systems::*;
use bevy_ecs::system::Resource;
use tokio::sync::oneshot;

#[derive(Debug, Default, Resource)]
pub struct Paused(bool);

impl Paused {
    pub fn set(&mut self, val: bool) {
        self.0 = val;
    }

    pub fn get(&self) -> bool {
        self.0
    }
}

#[derive(Debug, Resource)]
pub struct Shutdown {
    pub tx: Option<oneshot::Sender<()>>,
}
