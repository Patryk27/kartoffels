use crate::views::game::{BotSource, Event, Mode, UploadBotRequest, View};
use crate::{Button, Ui, UiWidget};
use ratatui::layout::{Constraint, Layout};
use termwiz::input::KeyCode;

#[derive(Debug, Default)]
pub struct IdleSidePanel;

impl IdleSidePanel {
    pub fn render(ui: &mut Ui<Event>, view: &View) {
        let btns = Self::btns(view);

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

    fn btns(view: &View) -> Vec<PanelButton> {
        let mut btns = Vec::new();

        match view.mode {
            Mode::Default => {
                if !view.config.hero_mode {
                    let btn = Button::new("join-bot", KeyCode::Char('j'))
                        .throwing(Event::OpenJoinBotModal);

                    btns.push(PanelButton {
                        btn,
                        enabled: !view.snapshot.bots.is_empty(),
                    });
                }

                if view.config.can_upload_bots {
                    let btn = Button::new("upload-bot", KeyCode::Char('u'))
                        .throwing(Event::OpenUploadBotModal {
                            request: UploadBotRequest::new(BotSource::Upload),
                        });

                    btns.push(PanelButton { btn, enabled: true });
                }

                if view.config.can_spawn_bots {
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
