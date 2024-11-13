use crate::{BotEvents, BotId, CreateBotRequest, QueuedBot, World};
use anyhow::{anyhow, Context, Result};
use kartoffels_cpu::Firmware;
use rand::Rng;
use tracing::debug;

pub fn run(world: &mut World, req: CreateBotRequest) -> Result<BotId> {
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

        if !req.instant {
            events.add("uploaded and queued");
        }

        events
    };

    let id = loop {
        let id = world.rng.gen();

        if !world.bots.contains(id) {
            break id;
        }
    };

    let fw = Firmware::from_elf(&src).context("couldn't parse firmware")?;

    let bot = QueuedBot {
        dir,
        events,
        fw,
        id,
        oneshot,
        pos,
        requeued: false,
        serial: Default::default(),
    };

    if instant {
        super::spawn::run_now(world, bot)?;
    } else {
        if world.bots.queued.len() >= world.policy.max_queued_bots {
            return Err(anyhow!(
                "too many robots queued, try again in a moment"
            ));
        }

        world.bots.queued.push(bot);
    }

    Ok(id)
}
