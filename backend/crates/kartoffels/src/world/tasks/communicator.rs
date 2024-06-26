use super::Broadcaster;
use crate::world::WorldRequest;
use crate::{AliveBot, World};
use kartoffels_vm as vm;
use rand::RngCore;
use tokio::sync::mpsc::Receiver;
use tracing::debug;

#[derive(Debug)]
pub struct Communicator {
    rx: Receiver<WorldRequest>,
}

impl Communicator {
    pub fn new(rx: Receiver<WorldRequest>) -> Self {
        Self { rx }
    }

    pub fn tick(
        &mut self,
        world: &mut World,
        rng: &mut impl RngCore,
        bcaster: &mut Broadcaster,
    ) {
        let Ok(msg) = self.rx.try_recv() else {
            return;
        };

        debug!(?msg, "processing message");

        match msg {
            WorldRequest::CreateBot { src, tx } => {
                _ = tx.send(
                    try {
                        let fw = vm::Firmware::new(&src)?;
                        let vm = vm::Runtime::new(fw);
                        let bot = AliveBot::new(rng, vm);

                        world.bots.add(rng, &world.policy, bot)?
                    },
                );
            }

            WorldRequest::Join { id, tx } => {
                _ = tx.send(bcaster.add(id));
            }
        }
    }
}
