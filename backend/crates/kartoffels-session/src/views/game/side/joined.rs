use crate::views::game::{Event, JoinedBot, State};
use crate::BotIdExt;
use kartoffels_ui::{theme, Button, RectExt, Render, Ui};
use kartoffels_world::prelude::{
    SnapshotAliveBot, SnapshotDeadBot, SnapshotQueuedBot,
};
use ordinal::Ordinal;
use ratatui::layout::Rect;
use ratatui::style::Stylize;
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
            2
        } else if state.perms.can_user_manage_bots {
            5
        } else {
            3
        };

        let bot_area = Rect {
            x: ui.area.x,
            y: ui.area.y,
            width: ui.area.width,
            height: ui.area.height - footer_height - 1,
        };

        ui.clamp(bot_area, |ui| {
            if let Some(bot) = state.snapshot.bots().alive().get(bot.id) {
                Self::render_alive_bot(ui, bot);
                return;
            }

            if let Some(bot) = state.snapshot.bots().dead().get(bot.id) {
                Self::render_dead_bot(ui, bot);
                return;
            }

            if let Some(bot) = state.snapshot.bots().queued().get(bot.id) {
                Self::render_queued_bot(ui, bot);
            }
        });

        ui.clamp(ui.area.footer(footer_height), |ui| {
            Self::render_footer(ui, state, bot)
        });
    }

    fn render_alive_bot(ui: &mut Ui<Event>, bot: &SnapshotAliveBot) {
        ui.line("status".underlined());
        ui.line("alive".fg(theme::GREEN));
        ui.line(format!("> age: {}s", bot.age).fg(theme::GRAY));
        ui.line(format!("> pos: {}", bot.pos).fg(theme::GRAY));
        ui.line(format!("> dir: {}", bot.dir).fg(theme::GRAY));
        ui.line(format!("> score: {}", bot.score).fg(theme::GRAY));
        ui.space(1);

        Self::render_bot_serial(ui, &bot.serial);
    }

    fn render_dead_bot(ui: &mut Ui<Event>, bot: &SnapshotDeadBot) {
        ui.line("status".underlined());
        ui.line("killed, discarded".fg(theme::RED));
        ui.space(1);

        Self::render_bot_serial(ui, &bot.serial);
    }

    fn render_queued_bot(ui: &mut Ui<Event>, bot: &SnapshotQueuedBot) {
        ui.line("status".underlined());

        ui.line(if bot.requeued {
            "killed, requeued".fg(theme::PINK)
        } else {
            "queued".fg(theme::PINK)
        });

        ui.line(format!("> place: {}", Ordinal(bot.place)).fg(theme::GRAY));
        ui.space(1);

        Self::render_bot_serial(ui, &bot.serial);
    }

    fn render_bot_serial(ui: &mut Ui<Event>, serial: &VecDeque<u32>) {
        ui.line("serial port".underlined());

        let serial = render_serial(serial);
        let serial = reflow_serial(&serial, ui.area);

        for line in serial {
            ui.line(line);
        }
    }

    // TODO refactor
    fn render_footer(ui: &mut Ui<Event>, state: &State, bot: &JoinedBot) {
        if state.perms.hero_mode {
            Button::new(KeyCode::Char('i'), "inspect-bot")
                .throwing(Event::InspectBot)
                .render(ui);

            if state.perms.can_user_manage_bots {
                Button::new(KeyCode::Char('D'), "delete-bot")
                    .throwing(Event::DeleteBot)
                    .render(ui);
            }
        } else {
            Button::new(KeyCode::Char('i'), "inspect-bot")
                .throwing(Event::InspectBot)
                .render(ui);

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

                    Button::new(KeyCode::Char('D'), "delete-bot")
                        .throwing(Event::DeleteBot)
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
