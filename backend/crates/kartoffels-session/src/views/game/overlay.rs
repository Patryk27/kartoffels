use super::{Event, Mode, State};
use kartoffels_ui::{theme, FromMarkdown, Ui};
use ratatui::style::Stylize;
use ratatui::text::Line;

#[derive(Debug)]
pub struct Overlay;

impl Overlay {
    pub fn render(ui: &mut Ui<Event>, state: &State) {
        match &state.mode {
            Mode::Default => {
                //
            }

            Mode::SpawningPrefabBot { prefab, .. } => {
                ui.with(|ui| {
                    ui.line(
                        Line::md(&format!(
                            "*left mouse button*: spawn {prefab}"
                        ))
                        .fg(theme::FG)
                        .bg(theme::BG),
                    );

                    ui.line(
                        Line::md("*esc*: stop spawning")
                            .fg(theme::FG)
                            .bg(theme::BG),
                    );
                });
            }
        }
    }
}
