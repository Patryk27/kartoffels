use super::WorldVis;
use kartoffels_world::prelude::Handle as WorldHandle;
use std::path::PathBuf;

#[derive(Debug)]
pub struct WorldEntry {
    pub vis: WorldVis,
    pub path: Option<PathBuf>,
    pub handle: WorldHandle,
}
