use crate::error::{AppError, AppResult};
use kartoffels::prelude::{Handle, WorldId, WorldName};
use std::collections::HashMap;
use std::mem;
use std::path::PathBuf;

#[derive(Debug)]
pub enum AppState {
    Alive(AliveAppState),
    ShuttingDown,
}

impl AppState {
    pub fn as_alive(&self) -> AppResult<&AliveAppState> {
        match self {
            AppState::Alive(state) => Ok(state),
            AppState::ShuttingDown => Err(AppError::ServerIsShuttingDown),
        }
    }

    pub fn as_alive_mut(&mut self) -> AppResult<&mut AliveAppState> {
        match self {
            AppState::Alive(state) => Ok(state),
            AppState::ShuttingDown => Err(AppError::ServerIsShuttingDown),
        }
    }

    pub fn take(&mut self) -> Option<AliveAppState> {
        let state = mem::replace(self, AppState::ShuttingDown);

        if let AppState::Alive(state) = state {
            Some(state)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct AliveAppState {
    pub data: Option<PathBuf>,
    pub worlds: HashMap<WorldId, Handle>,
}

impl AliveAppState {
    pub fn world(&self, id: WorldId) -> AppResult<Handle> {
        self.worlds.get(&id).ok_or(AppError::WorldNotFound).cloned()
    }

    pub fn contains_world_named(&self, name: &WorldName) -> bool {
        self.worlds.values().any(|world| world.name() == name)
    }
}
