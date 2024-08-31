use super::SidePanelResponse;
use crate::views::play::JoinedBot;
use crate::{BotIdExt, Ui};
use itertools::Either;
use kartoffels_world::prelude::{
    BotId, Snapshot, SnapshotAliveBot, SnapshotQueuedBot,
};
use ratatui::layout::{Constraint, Layout, Offset};
use ratatui::style::Stylize;
use ratatui::widgets::{Paragraph, Widget};

#[derive(Debug)]
pub struct JoinedSidePanel;

impl JoinedSidePanel {
    pub fn render(
        ui: &mut Ui,
        snapshot: &Snapshot,
        bot: &JoinedBot,
        _enabled: bool,
    ) -> Option<SidePanelResponse> {
        let [id_area, _, status_area, _, serial_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Length(2),
        ])
        .areas(ui.area());

        ui.clamp(id_area, |ui| {
            Self::render_id(ui, bot.id);
        });

        match snapshot.bots.by_id(bot.id) {
            Some(Either::Left(bot)) => {
                ui.clamp(status_area, |ui| {
                    Self::render_status_alive(ui, bot);
                });

                ui.clamp(serial_area, |ui| {
                    Self::render_serial(ui, bot);
                });
            }

            Some(Either::Right(bot)) => {
                ui.clamp(status_area, |ui| {
                    Self::render_status_queued(ui, bot);
                });
            }

            None => {
                // TODO
            }
        }

        None
    }

    fn render_id(ui: &mut Ui, id: BotId) {
        Paragraph::new("id").render(ui.area(), ui.buf());

        Paragraph::new(id.to_string().fg(id.color()))
            .render(ui.area().offset(Offset { x: 0, y: 1 }), ui.buf());
    }

    fn render_status_alive(ui: &mut Ui, bot: &SnapshotAliveBot) {
        Paragraph::new("status").render(ui.area(), ui.buf());

        Paragraph::new(format!("{} ({}s)", "alive".green(), bot.age))
            .render(ui.area().offset(Offset { x: 0, y: 1 }), ui.buf());
    }

    fn render_status_queued(ui: &mut Ui, bot: &SnapshotQueuedBot) {
        Paragraph::new("status").render(ui.area(), ui.buf());

        let status = if bot.requeued {
            format!("{} ({})", "requeued".magenta(), bot.place)
        } else {
            format!("{} ({})", "queued".magenta(), bot.place)
        };

        Paragraph::new(status)
            .render(ui.area().offset(Offset { x: 0, y: 1 }), ui.buf());
    }

    fn render_serial(ui: &mut Ui, bot: &SnapshotAliveBot) {
        Paragraph::new("serial port").render(ui.area(), ui.buf());

        Paragraph::new(bot.serial.as_str())
            .render(ui.area().offset(Offset { x: 0, y: 1 }), ui.buf());
    }
}
