use crate::views::play::{Event, JoinedBot, State};
use crate::BotIdExt;
use itertools::Either;
use kartoffels_ui::{theme, Button, RectExt, Ui};
use kartoffels_world::prelude::{SnapshotAliveBot, SnapshotQueuedBot};
use ratatui::layout::Rect;
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Widget};
use std::collections::VecDeque;
use termwiz::input::KeyCode;

#[derive(Debug)]
pub struct JoinedSidePanel;

impl JoinedSidePanel {
    pub fn render(ui: &mut Ui, state: &State, bot: &JoinedBot) {
        ui.line("id".underlined());
        ui.line(bot.id.to_string().fg(bot.id.color()));
        ui.space(1);

        let footer_height = if state.perms.single_bot_mode {
            1
        } else if state.perms.user_can_manage_bots {
            4
        } else {
            2
        };

        match state.snapshot.bots().by_id(bot.id) {
            Some(Either::Left(bot)) => {
                Self::render_alive_bot(ui, bot, footer_height);
            }
            Some(Either::Right(bot)) => {
                Self::render_queued_bot(ui, bot);
            }
            None => (),
        }

        ui.clamp(ui.area().footer(footer_height), |ui| {
            Self::render_footer(ui, state, bot)
        });
    }

    fn render_alive_bot(
        ui: &mut Ui,
        bot: &SnapshotAliveBot,
        footer_height: u16,
    ) {
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
                height: area.height - footer_height - 1,
            }
        };

        ui.clamp(area, |ui| {
            ui.line("serial port".underlined());

            Paragraph::new(render_serial(&bot.serial))
                .wrap(Default::default())
                .render(ui.area(), ui.buf());
        });
    }

    fn render_queued_bot(ui: &mut Ui, bot: &SnapshotQueuedBot) {
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

    fn render_footer(ui: &mut Ui, state: &State, bot: &JoinedBot) {
        if state.perms.single_bot_mode {
            if state.perms.user_can_manage_bots {
                Button::new(KeyCode::Char('D'), "destroy")
                    .throwing(Event::DestroyBot)
                    .render(ui);
            }
        } else {
            let follow_caption = if bot.is_followed {
                "stop following"
            } else {
                "follow"
            };

            Button::new(KeyCode::Char('f'), follow_caption)
                .throwing(Event::FollowBot)
                .render(ui);

            // TODO
            if false {
                Button::new(KeyCode::Char('i'), "history")
                    .throwing(Event::ShowBotHistoryDialog)
                    .render(ui);
            }

            if state.perms.user_can_manage_bots {
                ui.enable(!state.paused, |ui| {
                    Button::new(KeyCode::Char('R'), "restart")
                        .throwing(Event::RestartBot)
                        .render(ui);

                    Button::new(KeyCode::Char('D'), "destroy")
                        .throwing(Event::DestroyBot)
                        .render(ui);
                });

                ui.space(2);
            }

            Button::new(KeyCode::Char('l'), "leave")
                .throwing(Event::LeaveBot)
                .render(ui);
        }
    }
}

// TODO consider memoization, idk
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
