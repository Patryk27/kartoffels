use super::SidePanelEvent;
use crate::views::play::JoinedBot;
use crate::BotIdExt;
use itertools::Either;
use kartoffels_world::prelude::{
    BotId, Snapshot, SnapshotAliveBot, SnapshotQueuedBot,
};
use ratatui::layout::{Constraint, Layout, Offset};
use ratatui::prelude::{Buffer, Rect};
use ratatui::style::Stylize;
use ratatui::widgets::{Paragraph, Widget};
use termwiz::input::{InputEvent, KeyCode, Modifiers};

#[derive(Debug)]
pub struct JoinedSidePanel<'a> {
    pub snapshot: &'a Snapshot,
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

        match self.snapshot.bots.by_id(self.bot.id) {
            Some(Either::Left(bot)) => {
                Self::render_status_alive(status_area, buf, bot);
                Self::render_serial(serial_area, buf, bot);
            }

            Some(Either::Right(bot)) => {
                Self::render_status_queued(status_area, buf, bot);
            }

            None => {
                // TODO
            }
        }
    }

    fn render_id(area: Rect, buf: &mut Buffer, id: BotId) {
        Paragraph::new("id").render(area, buf);

        Paragraph::new(id.to_string().fg(id.color()))
            .render(area.offset(Offset { x: 0, y: 1 }), buf);
    }

    fn render_status_alive(
        area: Rect,
        buf: &mut Buffer,
        bot: &SnapshotAliveBot,
    ) {
        Paragraph::new("status").render(area, buf);

        Paragraph::new(format!("{} ({}s)", "alive".green(), bot.age))
            .render(area.offset(Offset { x: 0, y: 1 }), buf);
    }

    fn render_status_queued(
        area: Rect,
        buf: &mut Buffer,
        bot: &SnapshotQueuedBot,
    ) {
        Paragraph::new("status").render(area, buf);

        let status = if bot.requeued {
            format!("{} ({})", "requeued".magenta(), bot.place)
        } else {
            format!("{} ({})", "queued".magenta(), bot.place)
        };

        Paragraph::new(status).render(area.offset(Offset { x: 0, y: 1 }), buf);
    }

    fn render_serial(area: Rect, buf: &mut Buffer, bot: &SnapshotAliveBot) {
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
