use super::SidePanelResponse;
use crate::views::play::JoinedBot;
use crate::{theme, BotIdExt, Button, Ui};
use itertools::Either;
use kartoffels_world::prelude::Snapshot;
use ratatui::style::Stylize;
use termwiz::input::KeyCode;

#[derive(Debug)]
pub struct JoinedSidePanel;

impl JoinedSidePanel {
    pub fn render(
        ui: &mut Ui,
        snapshot: &Snapshot,
        bot: &JoinedBot,
        _enabled: bool,
    ) -> Option<SidePanelResponse> {
        ui.line("id");
        ui.line(bot.id.to_string().fg(bot.id.color()));

        match snapshot.bots.by_id(bot.id) {
            Some(Either::Left(bot)) => {
                ui.line("status");
                ui.text("alive".fg(theme::GREEN));
                ui.line(format!("({}s)", bot.age));
                ui.step(1);

                ui.line("serial port");
                ui.line(bot.serial.as_str());
                ui.step(1);
            }

            Some(Either::Right(bot)) => {
                ui.line("status");

                if bot.requeued {
                    ui.line("queued".fg(theme::PINK));
                } else {
                    ui.line("requeued".fg(theme::PINK));
                }

                ui.step(1);

                ui.line("serial port");
                ui.line(bot.serial.as_str());
                ui.step(1);
            }

            None => {
                ui.line("status");
                ui.line("dead".fg(theme::RED));
                ui.step(1);
            }
        }

        let mut response = None;

        if Button::new(KeyCode::Char('h'), "history")
            .block()
            .render(ui)
            .pressed
        {
            response = Some(SidePanelResponse::ShowBotHistory);
        }

        if Button::new(KeyCode::Char('l'), "leave bot")
            .block()
            .render(ui)
            .pressed
        {
            response = Some(SidePanelResponse::LeaveBot);
        }

        response
    }
}
