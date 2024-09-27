pub mod challenges;
pub mod online;
pub mod sandbox;
pub mod tutorial;

mod prelude {
    pub(super) use crate::views::game::{HelpDialog, HelpDialogResponse};
    pub(super) use anyhow::Result;
    pub(super) use glam::{ivec2, uvec2};
    pub(super) use kartoffels_ui::{theme, Dialog, DialogButton, DialogLine};
    pub(super) use kartoffels_world::prelude::*;
    pub(super) use ratatui::style::Stylize;
    pub(super) use ratatui::text::Span;
    pub(super) use std::sync::LazyLock;
    pub(super) use std::task::Poll;
    pub(super) use std::time::Duration;
    pub(super) use termwiz::input::KeyCode;
    pub(super) use tokio::time;
}
