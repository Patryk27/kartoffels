use crate::views::game::Event;
use kartoffels_ui::{theme, Button, Render, Ui};
use ratatui::text::Line;
use termwiz::input::KeyCode;

#[derive(Debug, Default)]
pub struct MenuModal;

impl MenuModal {
    pub fn render(&mut self, ui: &mut Ui<Event>, can_restart: bool) {
        let height = if can_restart { 5 } else { 4 };

        ui.window(30, height, Some(" menu "), theme::YELLOW, |ui| {
            ui.line(Line::raw("whaddya want to do?").centered());
            ui.line("");

            Button::new(KeyCode::Char('l'), "leave game")
                .throwing(Event::GoBack)
                .centered()
                .render(ui);

            if can_restart {
                Button::new(KeyCode::Char('r'), "restart game")
                    .throwing(Event::Restart)
                    .centered()
                    .render(ui);
            }

            Button::new(KeyCode::Escape, "go back to game")
                .throwing(Event::CloseModal)
                .centered()
                .render(ui);
        });
    }
}
