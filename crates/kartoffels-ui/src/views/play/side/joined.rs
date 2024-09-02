use super::SidePanelResponse;
use crate::views::play::JoinedBot;
use crate::{theme, BotIdExt, Button, RectExt, Ui};
use itertools::Either;
use kartoffels_world::prelude::Snapshot;
use ratatui::layout::Rect;
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Widget};
use std::collections::VecDeque;
use termwiz::input::KeyCode;

#[derive(Debug)]
pub struct JoinedSidePanel;

impl JoinedSidePanel {
    const FOOTER_HEIGHT: u16 = 3;

    pub fn render(
        ui: &mut Ui,
        world: &Snapshot,
        bot: &JoinedBot,
        enabled: bool,
    ) -> Option<SidePanelResponse> {
        let mut resp = None;

        ui.line("id".underlined());
        ui.line(bot.id.to_string().fg(bot.id.color()));
        ui.space(1);

        match world.bots().by_id(bot.id) {
            Some(Either::Left(bot)) => {
                ui.line("status".underlined());
                ui.line(Line::from_iter([
                    "alive ".fg(theme::GREEN),
                    format!("({}s)", bot.age).into(),
                ]));
                ui.space(1);

                // ---

                let area = {
                    let area = ui.area();

                    Rect {
                        x: area.x,
                        y: area.y,
                        width: area.width,
                        height: area.height - Self::FOOTER_HEIGHT - 1,
                    }
                };

                ui.clamp(area, |ui| {
                    ui.line("serial port".underlined());

                    Paragraph::new(render_serial(&bot.serial))
                        .wrap(Default::default())
                        .render(ui.area(), ui.buf());
                });
            }

            Some(Either::Right(bot)) => {
                ui.line("status".underlined());

                ui.row(|ui| {
                    let status = if bot.requeued {
                        "killed, requeued"
                    } else {
                        "queued"
                    };

                    ui.span(status.fg(theme::PINK));
                    ui.span(format!(" ({})", bot.place));
                });
            }

            None => {
                ui.line("status".underlined());
                ui.line("killed, dead".fg(theme::RED));
            }
        }

        ui.clamp(ui.area().footer(Self::FOOTER_HEIGHT), |ui| {
            if Button::new(KeyCode::Char('f'), "follow")
                .enabled(enabled)
                .block()
                .render(ui)
                .pressed
            {
                todo!();
            }

            if Button::new(KeyCode::Char('h'), "history")
                .enabled(enabled)
                .block()
                .render(ui)
                .pressed
            {
                resp = Some(SidePanelResponse::ShowBotHistory);
            }

            if Button::new(KeyCode::Char('l'), "leave")
                .enabled(enabled)
                .block()
                .render(ui)
                .pressed
            {
                resp = Some(SidePanelResponse::LeaveBot);
            }
        });

        resp
    }
}

fn render_serial(serial: &VecDeque<u32>) -> String {
    let mut out = String::with_capacity(256);
    let mut buf = None;

    for &ch in serial {
        match ch {
            0xffffff00 => {
                buf = Some(String::with_capacity(256));
            }

            0xffffff01 => {
                out = buf.take().unwrap_or_default();
            }

            ch => {
                if let Some(ch) = char::from_u32(ch) {
                    if let Some(buf) = &mut buf {
                        buf.push(ch);
                    } else {
                        out.push(ch);
                    }
                }
            }
        }
    }

    out
}
