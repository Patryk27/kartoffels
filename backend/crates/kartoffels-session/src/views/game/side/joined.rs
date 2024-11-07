use crate::views::game::{Event, JoinedBot, State};
use crate::BotIdExt;
use itertools::Either;
use kartoffels_ui::{theme, Button, RectExt, Render, Ui};
use kartoffels_world::prelude::{SnapshotAliveBot, SnapshotQueuedBot};
use ordinal::Ordinal;
use ratatui::layout::Rect;
use ratatui::style::Stylize;
use ratatui::text::Line;
use std::collections::VecDeque;
use termwiz::input::KeyCode;

#[derive(Debug)]
pub struct JoinedSidePanel;

impl JoinedSidePanel {
    pub fn render(ui: &mut Ui<Event>, state: &State, bot: &JoinedBot) {
        ui.line("id".underlined());
        ui.line(bot.id.to_string().fg(bot.id.color()));
        ui.space(1);

        let footer_height = if state.perms.hero_mode {
            1
        } else if state.perms.can_user_manage_bots {
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

        ui.clamp(ui.area.footer(footer_height), |ui| {
            Self::render_footer(ui, state, bot)
        });
    }

    fn render_alive_bot(
        ui: &mut Ui<Event>,
        bot: &SnapshotAliveBot,
        footer_height: u16,
    ) {
        ui.line("status".underlined());
        ui.line(Line::from_iter([
            "alive ".fg(theme::GREEN),
            format!("({}s)", bot.age).into(),
        ]));
        ui.line(format!("> score: {}", bot.score).fg(theme::GRAY));
        ui.space(1);

        // ---

        let area = Rect {
            x: ui.area.x,
            y: ui.area.y,
            width: ui.area.width,
            height: ui.area.height - footer_height - 1,
        };

        ui.clamp(area, |ui| {
            ui.line("serial port".underlined());

            let serial = render_serial(&bot.serial);
            let serial = reflow_serial(&serial, ui.area);

            for line in serial {
                ui.line(line);
            }
        });
    }

    fn render_queued_bot(ui: &mut Ui<Event>, bot: &SnapshotQueuedBot) {
        ui.line("status".underlined());

        ui.row(|ui| {
            let status = if bot.requeued {
                "killed, requeued"
            } else {
                "queued"
            };

            ui.span(status.fg(theme::PINK));
            ui.span(format!(" ({})", Ordinal(bot.place)));
        });
    }

    fn render_footer(ui: &mut Ui<Event>, state: &State, bot: &JoinedBot) {
        if state.perms.hero_mode {
            if state.perms.can_user_manage_bots {
                Button::new(KeyCode::Char('D'), "destroy-bot")
                    .throwing(Event::DestroyBot)
                    .render(ui);
            }
        } else {
            let follow_caption = if bot.follow {
                "stop-following-bot"
            } else {
                "follow-bot"
            };

            Button::new(KeyCode::Char('f'), follow_caption)
                .throwing(Event::FollowBot)
                .render(ui);

            if state.perms.can_user_manage_bots {
                ui.enable(!state.paused, |ui| {
                    Button::new(KeyCode::Char('R'), "restart-bot")
                        .throwing(Event::RestartBot)
                        .render(ui);

                    Button::new(KeyCode::Char('D'), "destroy-bot")
                        .throwing(Event::DestroyBot)
                        .render(ui);
                });

                ui.space(2);
            }

            Button::new(KeyCode::Char('l'), "leave-bot")
                .throwing(Event::LeaveBot)
                .render(ui);
        }
    }
}

// TODO this should be done by BotSerial and memoized
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

fn reflow_serial(serial: &str, area: Rect) -> VecDeque<&str> {
    let mut lines = VecDeque::with_capacity(area.height as usize);

    let mut line_start = 0;
    let mut line_chars = 0;

    // TODO we should iterate through graphemes here
    for (ch_idx, ch) in serial.char_indices() {
        if ch == '\n' || line_chars == area.width {
            if lines.len() == lines.capacity() {
                lines.pop_front();
            }

            lines.push_back(&serial[line_start..ch_idx]);

            line_start = ch_idx;
            line_chars = 0;

            if ch == '\n' {
                line_start += 1;
                continue;
            }
        }

        line_chars += 1;
    }

    if line_chars > 0 {
        if lines.len() == lines.capacity() {
            lines.pop_front();
        }

        lines.push_back(&serial[line_start..]);
    }

    lines
}
