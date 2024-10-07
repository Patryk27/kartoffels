use crate::{BotEvents, BotId, Event, QueuedBot, World};
use anyhow::{anyhow, Context, Result};
use glam::IVec2;
use kartoffels_cpu::Cpu;
use std::borrow::Cow;
use std::sync::Arc;
use tracing::info;

pub fn run(
    world: &mut World,
    src: Cow<'static, [u8]>,
    pos: Option<IVec2>,
) -> Result<BotId> {
    let cpu = Cpu::new(&src).context("couldn't parse firmware")?;
    let src_hash = sha256::digest(&src[..]);

    info!(?src_hash, "creating bot");

    let id = loop {
        let id = BotId::new(&mut world.rng);

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
            id,
            pos,
            cpu,
            events,
            serial: Default::default(),
            requeued: false,
        });

        _ = world.events.send(Arc::new(Event::BotCreated { id }));

        Ok(id)
    } else {
        Err(anyhow!("too many robots queued, try again in a moment"))
    }
}
