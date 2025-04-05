use std::collections::HashSet;

use crate::{BotId, Bots, WorldRng};
use ahash::{HashSet, HashSetExt};
use bevy_ecs::system::{ResMut, Resource};
use glam::IVec2;
use rand::{Rng, RngCore};

// Notes about the message system
// Mangling needs to be written
// But a basic implementation currently would mean that a message intersecting
// on one side of the radius would mangle the entire message, this would act weird
// a solution to this is each location contains it's own message.
// This brings us neatly into the expand function, which currently I am unhappy
// with the implementation and will need to be reworked especially if the messages
// are location dependent in which case each location will need to calculate their
// next positions which might overlap maybe this overlapping would do some sort of
// error correction by mangling the message *back*
//
// Also messages can currently hit the same bot multiple times, to reduce this I
// only do the check when it expands but there is a chance that the message tick
// and bot movement intersect in such a way that a message is missed, currently
// this is band-aid fixed by keeping a list of all the bots the message has hit
// and just not sending them a new copy of the message, but this isn't very realisitic
//
pub fn tick(
    mut messages: ResMut<Messages>,
    bots: ResMut<Bots>,
    mut rng: ResMut<WorldRng>,
) {
    let mut msgs_to_delete: Vec<usize> = Vec::new();

    for (msg_idx, msg) in messages.entries.iter_mut().enumerate() {
        // Expand out the message radius
        if msg.expand_cooldown == 0 {
            if msg.curr_radius == msg.strength {
                msgs_to_delete.push(msg_idx)
            }
            msg.expand();
            msg.bit_decay(msg.curr_radius as f64, &mut rng.0);
            msg.expand_cooldown = 5_000; // Messages expand out ever ~ 5000 ticks
        }
        msg.expand_cooldown = msg.expand_cooldown.saturating_sub(1);

        // Check if the message has hit any bots
        for location in &msg.locations {
            if let Some(id) = bots.alive.lookup_at(*location) {
                if msg.hit_bots.contains(&id) {
                    continue;
                }
                msg.hit_bots.insert(id);
                let _ = bots
                    .alive
                    .get(id)
                    .unwrap()
                    .radio
                    .receive_message(&msg.content);
            }
        }
        // Check if the message has intersected with any others and run mangle messages
    }

    messages.clean_messages(msgs_to_delete);
}

fn mangle_messages(left: &mut Message, right: &mut Message) {
    // if two messages *collide* they should interfere with eachother, this will
    // likely be impacted by strength as well
    todo!()
}

/// This is the storage medium for all the messages in the world
#[derive(Clone, Debug, Default, Resource)]
pub struct Messages {
    entries: Vec<Message>,
}

impl Messages {
    pub fn add_message(
        &mut self,
        content: &[u8],
        source: IVec2,
        strength: i32,
    ) {
        let new_message = Message {
            content: Vec::from(content),
            source,
            strength,
            curr_radius: 0,
            locations: HashSet::new(),
            expand_cooldown: 5000,
            hit_bots: HashSet::new(),
        };
        self.entries.push(new_message);
    }

    fn clean_messages(&mut self, del_list: Vec<usize>) {
        for (index, id) in del_list.iter().enumerate() {
            self.entries.remove(id - index);
        }
    }
}

/// This is the in-world representation of a radio message
#[derive(Clone, Debug)]
pub struct Message {
    content: Vec<u8>,
    source: IVec2,
    strength: i32,
    curr_radius: i32,
    locations: HashSet<IVec2>,
    expand_cooldown: usize,
    hit_bots: HashSet<BotId>,
}

impl Message {
    fn bit_decay(&mut self, prob: f64, rng: &mut impl RngCore) {
        // randomly flip bits throughout the message, prob will increase for flight
        // time to avoid the first 4 bytes (often used for filtering) from being
        // flipped as much the probability of each bit flipping also increases
        // with it's index within the messages
        // Currently it actually just runs an XOR on the bytes meaning that bytes decay really not bits
        for (idx, byte) in self.content.iter_mut().enumerate() {
            let bit_prob = (idx as f64 / 128.0).clamp(0.0, 1.0) * prob;
            if rng.gen_bool(bit_prob) {
                let rand: u8 = rng.gen();
                *byte ^= rand;
            }
        }
    }

    /// This is an ineffcient algorithm for calculating the positions of all the message waves
    /// TODO: Come back to this and make it nicer
    fn expand(&mut self) {
        let mut new_set: HashSet<IVec2> = HashSet::new();
        let new_radius = self.curr_radius + 1;
        let off = self.source - IVec2::new(new_radius, new_radius);
        for column in 0..=new_radius {
            new_set.insert(off + IVec2::new(column, new_radius + column));
            new_set.insert(off + IVec2::new(column, new_radius - column));
            new_set.insert(
                off + IVec2::new(
                    (2 * new_radius) - column,
                    new_radius + column,
                ),
            );
            new_set.insert(
                off + IVec2::new(
                    (2 * new_radius) - column,
                    new_radius - column,
                ),
            );
        }

        self.locations = new_set;
        self.curr_radius = new_radius;
    }
}
