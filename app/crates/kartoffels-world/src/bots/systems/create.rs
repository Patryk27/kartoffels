use crate::{
    BotEvents, Bots, Clock, CreateBot, CreateBotRequest, Policy, QueuedBot,
    SpawnBot, WorldRng,
};
use anyhow::{anyhow, Context};
use bevy_ecs::event::EventMutator;
use bevy_ecs::system::{Commands, Res, ResMut};
use kartoffels_cpu::Firmware;
use rand::Rng;
use tracing::debug;

pub fn create(
    mut cmds: Commands,
    mut bots: ResMut<Bots>,
    clock: Res<Clock>,
    policy: Res<Policy>,
    mut rng: ResMut<WorldRng>,
    mut events: EventMutator<CreateBot>,
) {
    for event in events.read() {
        let (Some(req), Some(tx)) = (event.req.take(), event.tx.take()) else {
            continue;
        };

        let CreateBotRequest {
            src,
            pos,
            dir,
            instant,
            oneshot,
        } = req;

        debug!(
            src = ?sha256::digest(&src[..])[0..8],
            ?pos,
            ?dir,
            ?instant,
            ?oneshot,
            "creating bot",
        );

        let events = {
            let mut events = BotEvents::default();

            if !instant {
                events.add(&clock, "uploaded");
            }

            events
        };

        let id = loop {
            let id = rng.0.gen();

            if !bots.contains(id) {
                break id;
            }
        };

        let fw = match Firmware::from_elf(&src) {
            Ok(fw) => fw,

            Err(err) => {
                _ = tx.send(Err(err).context("couldn't parse firmware"));
                continue;
            }
        };

        let bot = Box::new(QueuedBot {
            dir,
            events,
            fw,
            id,
            oneshot,
            pos,
            requeued: false,
            serial: Default::default(),
        });

        if instant {
            cmds.send_event(SpawnBot {
                bot: Some(bot),
                tx: Some(tx),
                requeue_if_cant_spawn: false,
            });
        } else {
            if bots.queued.len() >= policy.max_queued_bots as usize {
                _ = tx.send(Err(anyhow!(
                    "too many bots queued, try again in a moment"
                )));

                continue;
            }

            bots.queued.push_back(bot);

            _ = tx.send(Ok(id));
        }
    }
}
