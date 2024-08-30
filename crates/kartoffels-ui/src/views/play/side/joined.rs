use super::SidePanelEvent;
use crate::views::play::JoinedBot;
use crate::BotIdExt;
use kartoffels_world::prelude::{BotId, Update, UpdateBot, UpdateBotStatus};
use ratatui::layout::{Constraint, Layout, Offset};
use ratatui::prelude::{Buffer, Rect};
use ratatui::style::Stylize;
use ratatui::widgets::{Paragraph, Widget};
use termwiz::input::{InputEvent, KeyCode, Modifiers};

#[derive(Debug)]
pub struct JoinedSidePanel<'a> {
    pub update: &'a Update,
    pub bot: &'a JoinedBot,
    pub enabled: bool,
}

impl<'a> JoinedSidePanel<'a> {
    pub fn render(self, area: Rect, buf: &mut Buffer) {
        let [id_area, _, status_area, _, serial_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Length(2),
        ])
        .areas(area);

        Self::render_id(id_area, buf, self.bot.id);

        let Some(bot) = self.update.bots.by_id(self.bot.id) else {
            return;
        };

        Self::render_status(status_area, buf, bot);
        Self::render_serial(serial_area, buf, bot);
    }

    fn render_id(area: Rect, buf: &mut Buffer, id: BotId) {
        Paragraph::new("id").render(area, buf);

        Paragraph::new(id.to_string().fg(id.color()))
            .render(area.offset(Offset { x: 0, y: 1 }), buf);
    }

    fn render_status(area: Rect, buf: &mut Buffer, bot: &UpdateBot) {
        Paragraph::new("status").render(area, buf);

        let status = match &bot.status {
            UpdateBotStatus::Alive { age } => {
                format!("{} ({}s)", "alive".green(), age)
            }

            UpdateBotStatus::Queued { place, requeued } => {
                if *requeued {
                    format!("{} ({})", "requeued".magenta(), place)
                } else {
                    format!("{} ({})", "queued".magenta(), place)
                }
            }
        };

        Paragraph::new(status).render(area.offset(Offset { x: 0, y: 1 }), buf);
    }

    fn render_serial(area: Rect, buf: &mut Buffer, bot: &UpdateBot) {
        Paragraph::new("serial port").render(area, buf);

        Paragraph::new(bot.serial.as_str())
            .render(area.offset(Offset { x: 0, y: 1 }), buf);
    }

    pub fn handle(event: InputEvent) -> SidePanelEvent {
        if let InputEvent::Key(event) = &event {
            if event.key == KeyCode::Char('l')
                && event.modifiers == Modifiers::NONE
            {
                return SidePanelEvent::LeaveBot;
            }
        }

        SidePanelEvent::Forward(event)
    }
}
