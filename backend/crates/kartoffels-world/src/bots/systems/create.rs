use crate::{BotEvents, BotId, CreateBotRequest, Event, QueuedBot, World};
use anyhow::{anyhow, Context, Result};
use kartoffels_cpu::Firmware;
use rand::Rng;
use std::sync::Arc;
use tracing::info;

pub fn run(
    world: &mut World,
    CreateBotRequest {
        src,
        pos,
        dir,
        oneshot,
    }: CreateBotRequest,
) -> Result<BotId> {
    info!(src = ?sha256::digest(&src[..]), "creating bot");

    let fw = Firmware::from_elf(&src).context("couldn't parse firmware")?;

    let id = loop {
        let id = world.rng.gen();

        if !world.bots.contains(id) {
            break id;
        }
    };

    if world.bots.queued.len() < world.policy.max_queued_bots {
        let events = {
            let mut events = BotEvents::default();

            events.add("uploaded and queued");
            events
        };

        world.bots.queued.push(QueuedBot {
            dir,
            events,
            fw,
            id,
            oneshot,
            pos,
            requeued: false,
            serial: Default::default(),
        });

        _ = world.events.send(Arc::new(Event::BotCreated { id }));

        Ok(id)
    } else {
        Err(anyhow!("too many robots queued, try again in a moment"))
    }
}
