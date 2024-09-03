use std::future;
use kartoffels_world::prelude::Handle as WorldHandle;

#[derive(Debug)]
pub enum Mode {
    Normal(WorldHandle),
    Sandbox(WorldHandle),
    Challenge(kartoffels_challenges::EventRx),
}

impl Mode {
    pub fn is_sandbox(&self) -> bool {
        matches!(self, Mode::Sandbox(_))
    }

    pub async fn recv(&mut self) -> Option<kartoffels_challenges::Event> {
        if let Mode::Challenge(this) = self {
            this.recv().await
        } else {
            future::pending().await
        }
    }
}
