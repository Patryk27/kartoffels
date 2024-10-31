pub mod challenges;
pub mod online;
pub mod sandbox;
pub mod tutorial;
mod utils;

mod prelude {
    pub(super) use crate::drivers::utils;
    pub(super) use crate::game::Perms;
    pub(super) use crate::views::game::{HelpDialog, HelpDialogResponse};
    pub(super) use crate::DrivenGame;
    pub(super) use anyhow::Result;
    pub(super) use futures_util::future::BoxFuture;
    pub(super) use glam::{ivec2, uvec2, UVec2};
    pub(super) use kartoffels_store::Store;
    pub(super) use kartoffels_ui::{theme, Dialog, DialogButton, DialogLine};
    pub(super) use kartoffels_world::prelude::*;
    pub(super) use rand::{Rng, RngCore, SeedableRng};
    pub(super) use rand_chacha::ChaCha8Rng;
    pub(super) use ratatui::style::Stylize;
    pub(super) use ratatui::text::Span;
    pub(super) use std::future;
    pub(super) use std::sync::LazyLock;
    pub(super) use std::time::Duration;
    pub(super) use termwiz::input::KeyCode;
    pub(super) use tokio::sync::mpsc;
    pub(super) use tokio::time;
}
