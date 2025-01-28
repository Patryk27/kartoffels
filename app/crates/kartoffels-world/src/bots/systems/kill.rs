use crate::{Bots, Clock, DeadBot, Event, KillBot, Policy, QueuedBot};
use bevy_ecs::event::EventMutator;
use bevy_ecs::system::{Commands, Res, ResMut};
use tracing::trace;

pub fn kill(
    mut cmds: Commands,
    clock: Res<Clock>,
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

        let mut killed = *killed
            .take()
            .expect("bot is missing - maybe event has been already processed");

        trace!(id=?killed.id, ?reason, ?killer, "killing bot");

        cmds.send_event(Event::BotDied {
            id: killed.id,
            age: killed.age(),
        });

        if let Some(id) = killer {
            cmds.send_event(Event::BotScored { id: *id });
        }

        killed.log(&clock, &*reason);

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
                killed.log(&clock, "awaiting reincarnation");

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
