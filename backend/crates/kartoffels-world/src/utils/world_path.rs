use bevy_ecs::system::Resource;
use std::path::PathBuf;

#[derive(Debug, Resource)]
pub struct WorldPath(pub PathBuf);
