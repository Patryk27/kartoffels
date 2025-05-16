use crate::views::game::{BotSource, Event, Mode, State, UploadBotRequest};
use crate::{Button, Ui, UiWidget};
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

        ui.at(area, |ui| {
            for PanelButton { btn, enabled } in btns {
                ui.enabled(enabled, |ui| {
                    btn.render(ui);
                });
            }
        });
    }

    fn btns(state: &State) -> Vec<PanelButton> {
        let mut btns = Vec::new();

        match state.mode {
            Mode::Default => {
                if !state.config.hero_mode {
                    let btn = Button::new("join-bot", KeyCode::Char('j'))
                        .throwing(Event::OpenJoinBotModal);

                    btns.push(PanelButton {
                        btn,
                        enabled: !state.snapshot.bots.is_empty(),
                    });
                }

                if state.config.can_upload_bots {
                    let btn = Button::new("upload-bot", KeyCode::Char('u'))
                        .throwing(Event::OpenUploadBotModal {
                            request: UploadBotRequest::new(BotSource::Upload),
                        });

                    btns.push(PanelButton { btn, enabled: true });
                }

                if state.config.can_spawn_bots {
                    let btn = Button::new("spawn-bot", KeyCode::Char('S'))
                        .throwing(Event::OpenSpawnBotModal);

                    btns.push(PanelButton { btn, enabled: true });
                }
            }

            Mode::SpawningBot { .. } => {
                //
            }
        }

        btns
    }
}

struct PanelButton {
    btn: Button<'static, Event>,
    enabled: bool,
}
