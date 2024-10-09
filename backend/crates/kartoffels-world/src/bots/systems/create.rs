use crate::{BotEvents, BotId, CreateBotRequest, Event, QueuedBot, World};
use anyhow::{anyhow, Context, Result};
use kartoffels_cpu::Cpu;
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
    let cpu = Cpu::new(&src).context("couldn't parse firmware")?;
    let src_hash = sha256::digest(&src[..]);

    info!(?src_hash, "creating bot");

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
            cpu,
            dir,
            events,
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
