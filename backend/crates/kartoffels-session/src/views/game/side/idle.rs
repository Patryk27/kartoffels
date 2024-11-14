use crate::views::game::{Event, Mode, State, UploadBotRequest};
use kartoffels_ui::{Button, Render, Ui};
use ratatui::layout::{Constraint, Layout};
use termwiz::input::KeyCode;

#[derive(Debug, Default)]
pub struct IdleSidePanel;

impl IdleSidePanel {
    pub fn render(ui: &mut Ui<Event>, state: &State) {
        let btns = Self::btns(state);

        let [_, area] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(btns.len() as u16),
        ])
        .areas(ui.area);

        ui.clamp(area, |ui| {
            for btn in btns {
                btn.render(ui);
            }
        });
    }

    fn btns(state: &State) -> Vec<Button<Event>> {
        let mut btns = Vec::new();

        match state.mode {
            Mode::Default => {
                if !state.config.hero_mode {
                    btns.push(
                        Button::new(KeyCode::Char('j'), "join-bot")
                            .throwing(Event::OpenJoinBotModal)
                            .enabled(!state.snapshot.bots().is_empty()),
                    );
                }

                if state.config.can_upload_bots {
                    btns.push(
                        Button::new(KeyCode::Char('u'), "upload-bot").throwing(
                            Event::OpenUploadBotModal {
                                request: UploadBotRequest::default(),
                            },
                        ),
                    );
                }

                if state.config.can_spawn_bots {
                    btns.push(
                        Button::new(KeyCode::Char('S'), "spawn-bot")
                            .throwing(Event::OpenSpawnBotModal),
                    );
                }
            }

            Mode::SpawningBot { .. } => {
                //
            }
        }

        btns
    }
}
