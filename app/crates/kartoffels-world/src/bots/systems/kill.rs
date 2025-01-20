use crate::{Bots, DeadBot, Event, KillBot, Policy, QueuedBot};
use bevy_ecs::event::EventMutator;
use bevy_ecs::system::{Commands, Res, ResMut};
use itertools::Either;
use tracing::trace;

pub fn kill(
    mut cmds: Commands,
    policy: Res<Policy>,
    mut bots: ResMut<Bots>,
    mut events: EventMutator<KillBot>,
) {
    for event in events.read() {
        let KillBot {
            killed,
            reason,
            killer,
        } = event;

        match &killed {
            Either::Left(id) => {
                trace!(killed=?id, ?reason, ?killer, "killing bot");
            }
            Either::Right(bot) => {
                trace!(killed=?bot, ?reason, ?killer, "killing bot");
            }
        }

        let mut killed = match killed {
            Either::Left(id) => {
                let Some(bot) = bots.alive.remove(*id) else {
                    // Mildly sus, but not fatal - can happen when user tries to
                    // restart a queued bot
                    continue;
                };

                bot
            }

            Either::Right(bot) => *bot.take().expect(
                "bot is missing - maybe event has been already processed",
            ),
        };

        cmds.send_event(Event::BotDied {
            id: killed.id,
            age: killed.age(),
        });

        if let Some(id) = killer {
            cmds.send_event(Event::BotScored { id: *id });
        }

        killed.log(&*reason);

        let decision = if !killed.oneshot
            && policy.auto_respawn
            && bots.queued.len() < policy.max_queued_bots
        {
            Decision::Requeue
        } else {
            Decision::Discard
        };

        match decision {
            Decision::Requeue => {
                killed.log("awaiting reincarnation");

                bots.queued.push_back(Box::new(QueuedBot {
                    dir: None,
                    events: killed.events,
                    fw: killed.fw,
                    id: killed.id,
                    oneshot: false,
                    pos: None,
                    requeued: true,
                    serial: killed.serial,
                }));
            }

            Decision::Discard => {
                let bot = DeadBot {
                    events: killed.events.snapshot(),
                    id: killed.id,
                    serial: killed.serial.snapshot(),
                };

                if let Some(id) = bots.dead.add(bot) {
                    cmds.send_event(Event::BotDiscarded { id });
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Decision {
    Requeue,
    Discard,
}
