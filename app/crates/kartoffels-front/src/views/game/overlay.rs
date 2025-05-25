use super::{Event, Mode, View};
use crate::{FromMarkdown, Ui, UiWidget, theme};
use kartoffels_store::Store;
use kartoffels_world::prelude as w;
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};

#[derive(Debug)]
pub struct Overlay;

impl Overlay {
    pub fn render(ui: &mut Ui<Event>, store: &Store, view: &View) {
        // When testing, print the snapshot version - this makes assertions
        // more reliable because the test can simply say "wait until we see
        // v123" on the screen
        if store.testing()
            && let w::Clock::Manual { .. } = view.snapshot.clock
        {
            Span::raw(format!("v{}", view.snapshot.version)).render(ui);
        }

        match &view.mode {
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
