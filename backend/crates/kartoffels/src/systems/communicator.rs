use super::Broadcaster;
use crate::{AliveBot, Request, World};
use kartoffels_vm as vm;
use rand::RngCore;
use tokio::sync::mpsc::Receiver;
use tracing::debug;

#[derive(Debug)]
pub struct Communicator {
    rx: Receiver<Request>,
}

impl Communicator {
    pub fn new(rx: Receiver<Request>) -> Self {
        Self { rx }
    }

    pub fn tick(
        &mut self,
        world: &mut World,
        rng: &mut impl RngCore,
        bcaster: &mut Broadcaster,
    ) {
        while let Ok(msg) = self.rx.try_recv() {
            debug!(?msg, "processing message");

            match msg {
                Request::Upload { src, tx } => {
                    _ = tx.send(
                        try {
                            let fw = vm::Firmware::new(&src)?;
                            let vm = vm::Runtime::new(fw);
                            let mut bot = AliveBot::new(rng, vm);

                            bot.log("uploaded and queued".into());
                            world.bots.add(rng, &world.policy, bot)?
                        },
                    );
                }

                Request::Join { id, tx } => {
                    _ = tx.send(bcaster.add(world, id));
                }
            }
        }
    }
}
