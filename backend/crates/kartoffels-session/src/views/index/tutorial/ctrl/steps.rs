pub mod step01;
pub mod step02;
pub mod step03;
pub mod step04;
pub mod step05;
pub mod step06;
pub mod step07;
pub mod step08;
pub mod step09;
pub mod step10;
pub mod step11;
pub mod step12;
pub mod step13;
pub mod step14;
pub mod step15;
pub mod step16;

mod prelude {
    pub(super) use super::super::TutorialCtxt;
    pub(super) use crate::views::game::{HelpMsg, HelpMsgResponse};
    pub(super) use anyhow::Result;
    pub(super) use glam::*;
    pub(super) use kartoffels_ui::{theme, Msg, MsgButton, MsgLine};
    pub(super) use kartoffels_world::prelude::*;
    pub(super) use ratatui::style::Stylize;
    pub(super) use ratatui::text::Span;
    pub(super) use std::sync::LazyLock;
    pub(super) use std::time::Duration;
    pub(super) use termwiz::input::KeyCode;
    pub(super) use tokio::time;
}