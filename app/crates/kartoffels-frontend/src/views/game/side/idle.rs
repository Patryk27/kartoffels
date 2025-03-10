use crate::views::game::{BotSource, Event, Mode, State, UploadBotRequest};
use kartoffels_ui::{Button, KeyCode, Ui, UiWidget};
use ratatui::layout::{Constraint, Layout};

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
                        Button::new("join-bot", KeyCode::Char('j'))
                            .throwing(Event::OpenJoinBotModal)
                            .enabled(!state.snapshot.bots.is_empty()),
                    );
                }

                if state.config.can_upload_bots {
                    btns.push(
                        Button::new("upload-bot", KeyCode::Char('u')).throwing(
                            Event::OpenUploadBotModal {
                                request: UploadBotRequest::new(
                                    BotSource::Upload,
                                ),
                            },
                        ),
                    );
                }

                if state.config.can_spawn_bots {
                    btns.push(
                        Button::new("spawn-bot", KeyCode::Char('S'))
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
