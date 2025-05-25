use crate::views::game::{Event, JoinedBot, View};
use crate::{BotIdExt, Button, Ui, UiWidget, theme};
use kartoffels_world::prelude as w;
use ordinal::Ordinal;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Stylize;
use std::collections::VecDeque;
use termwiz::input::KeyCode;

#[derive(Debug)]
pub struct JoinedSidePanel;

impl JoinedSidePanel {
    pub fn render(ui: &mut Ui<Event>, view: &View, jbot: &JoinedBot) {
        let bot = view.snapshot.bots.get(jbot.id);
        let btns = Self::btns(view, jbot);

        let [bot_area, _, btns_area] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Length(btns.len() as u16),
        ])
        .areas(ui.area);

        ui.at(bot_area, |ui| {
            Self::render_bot(ui, jbot, bot);
        });

        ui.at(btns_area, |ui| {
            for btn in btns {
                btn.render(ui);
            }
        });
    }

    fn render_bot(
        ui: &mut Ui<Event>,
        jbot: &JoinedBot,
        bot: Option<w::BotSnapshot>,
    ) {
        ui.line("id".underlined());
        ui.line(jbot.id.to_string().fg(jbot.id.color()));
        ui.space(1);

        match bot {
            Some(w::BotSnapshot::Alive(bot)) => {
                Self::render_alive_bot(ui, bot);
            }
            Some(w::BotSnapshot::Dead(bot)) => {
                Self::render_dead_bot(ui, bot);
            }
            Some(w::BotSnapshot::Queued(bot)) => {
                Self::render_queued_bot(ui, bot);
            }
            _ => (),
        }
    }

    fn render_alive_bot(ui: &mut Ui<Event>, bot: &w::AliveBotSnapshot) {
        ui.line("status".underlined());
        ui.line("alive".fg(theme::GREEN));
        ui.line(format!("> age: {}", bot.age.as_time(None)).fg(theme::GRAY));
        ui.line(format!("> pos: {},{}", bot.pos.x, bot.pos.y).fg(theme::GRAY));
        ui.line(format!("> dir: {}", bot.dir).fg(theme::GRAY));
        ui.line(format!("> score: {}", bot.score).fg(theme::GRAY));
        ui.space(1);

        Self::render_bot_serial(ui, &bot.serial);
    }

    fn render_dead_bot(ui: &mut Ui<Event>, bot: &w::DeadBotSnapshot) {
        ui.line("status".underlined());
        ui.line("dead".fg(theme::RED));
        ui.space(1);

        Self::render_bot_serial(ui, &bot.serial);
    }

    fn render_queued_bot(ui: &mut Ui<Event>, bot: &w::QueuedBotSnapshot) {
        ui.line("status".underlined());

        ui.line(if bot.reincarnated {
            "awaiting reincarnation".fg(theme::PINK)
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

    fn btns(view: &View, bot: &JoinedBot) -> Vec<Button<'static, Event>> {
        let mut btns = Vec::new();

        btns.push(
            Button::new("inspect-bot", KeyCode::Char('i'))
                .throwing(Event::InspectBot { id: bot.id }),
        );

        btns.push({
            let label = if bot.follow {
                "stop-following-bot"
            } else {
                "follow-bot"
            };

            Button::new(label, KeyCode::Char('f')).throwing(Event::FollowBot)
        });

        if view.config.can_kill_bots {
            btns.push(
                Button::new("kill-bot", KeyCode::Char('K'))
                    .throwing(Event::KillBot),
            );
        }

        if view.config.can_delete_bots {
            btns.push(
                Button::new("delete-bot", KeyCode::Char('D'))
                    .throwing(Event::DeleteBot),
            );
        }

        if !view.config.hero_mode {
            btns.push(
                Button::new("leave-bot", KeyCode::Char('l'))
                    .throwing(Event::LeaveBot),
            );
        }

        btns
    }
}

// TODO this should be done by BotSerial and memoized
fn render_serial(serial: &VecDeque<u32>) -> String {
    serial.iter().copied().filter_map(char::from_u32).collect()
}

// TODO this should be done by BotSerial and memoized
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
