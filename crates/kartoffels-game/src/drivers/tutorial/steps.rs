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

mod prelude {
    pub(super) use crate::drivers::tutorial::StepCtxt;
    pub(super) use crate::play::{HelpDialog, HelpDialogResponse};
    pub(super) use anyhow::Result;
    pub(super) use kartoffels_ui::{theme, Dialog, DialogButton, DialogLine};
    pub(super) use ratatui::style::Stylize;
    pub(super) use ratatui::text::Span;
    pub(super) use std::sync::LazyLock;
    pub(super) use std::time::Duration;
    pub(super) use termwiz::input::KeyCode;
    pub(super) use tokio::time;
}
