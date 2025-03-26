use bevy_ecs::system::{ResMut, Resource};
use glam::IVec2;

use crate::Bots;

pub fn tick(mut messages: ResMut<Messages>, mut bots: ResMut<Bots>) {
    // Each ~? tick do all the message operations
}

/// This is the storage medium for all the messages in the world
#[derive(Clone, Debug, Default, Resource)]
pub struct Messages {
    entries: Vec<Message>,
}

impl Messages {}

/// This is the in-world representation of a radio message
#[derive(Clone, Debug)]
pub struct Message {
    content: Vec<u8>,
    source: IVec2,
    strength: usize,
}

impl Message {}
