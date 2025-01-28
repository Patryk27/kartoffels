use super::{Event, Mode, State};
use kartoffels_store::Store;
use kartoffels_ui::{theme, FromMarkdown, Ui, UiWidget};
use kartoffels_world::prelude::Clock;
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};

#[derive(Debug)]
pub struct Overlay;

impl Overlay {
    pub fn render(ui: &mut Ui<Event>, store: &Store, state: &State) {
        if let Clock::Manual { .. } = state.snapshot.clock
            && store.testing()
        {
            Span::raw(format!("v{}", state.snapshot.version)).render(ui);
        }

        match &state.mode {
            Mode::Default => {
                //
            }

            Mode::SpawningBot { .. } => {
                ui.with(|ui| {
                    ui.line(
                        Line::md("*left mouse button*: spawn bot")
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
