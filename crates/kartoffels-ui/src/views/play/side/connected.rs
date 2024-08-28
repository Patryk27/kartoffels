use crate::BotIdExt;
use kartoffels_world::prelude::{BotId, BotStatusUpdate, BotUpdate};
use ratatui::layout::{Constraint, Layout, Offset};
use ratatui::prelude::{Buffer, Rect};
use ratatui::style::Stylize;
use ratatui::widgets::{Paragraph, Widget};
use termwiz::input::{InputEvent, KeyCode, Modifiers};

#[derive(Debug)]
pub struct ConnectedSidePanel {
    pub id: BotId,
}

impl ConnectedSidePanel {
    pub fn handle(&self, event: InputEvent) -> ConnectedSidePanelOutcome {
        if let InputEvent::Key(event) = &event {
            if event.key == KeyCode::Char('l')
                && event.modifiers == Modifiers::NONE
            {
                return ConnectedSidePanelOutcome::Disconnect;
            }
        }

        ConnectedSidePanelOutcome::Forward(event)
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer, bots: &[BotUpdate]) {
        let [id_area, _, status_area, _, serial_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Length(2),
        ])
        .areas(area);

        Self::render_id(id_area, buf, self.id);

        // TODO create a lookup table
        let Some(bot) = bots.iter().find(|bot| bot.id == self.id) else {
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

    fn render_status(area: Rect, buf: &mut Buffer, bot: &BotUpdate) {
        Paragraph::new("status").render(area, buf);

        let status = match &bot.status {
            BotStatusUpdate::Alive { age } => {
                format!("{} ({}s)", "alive".green(), age)
            }

            BotStatusUpdate::Queued { place, requeued } => {
                if *requeued {
                    format!("{} ({})", "requeued".magenta(), place)
                } else {
                    format!("{} ({})", "queued".magenta(), place)
                }
            }
        };

        Paragraph::new(status).render(area.offset(Offset { x: 0, y: 1 }), buf);
    }

    fn render_serial(area: Rect, buf: &mut Buffer, bot: &BotUpdate) {
        Paragraph::new("serial port").render(area, buf);

        Paragraph::new(bot.serial.as_str())
            .render(area.offset(Offset { x: 0, y: 1 }), buf);
    }
}

#[derive(Debug)]
pub enum ConnectedSidePanelOutcome {
    Disconnect,
    Forward(InputEvent),
}
