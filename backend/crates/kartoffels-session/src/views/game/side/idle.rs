use crate::views::game::{Event, Mode, State, UploadBotRequest};
use kartoffels_ui::{Button, Render, Ui};
use ratatui::layout::{Constraint, Layout};
use termwiz::input::KeyCode;

#[derive(Debug, Default)]
pub struct IdleSidePanel;

impl IdleSidePanel {
    pub fn render(ui: &mut Ui<Event>, state: &State) {
        let actions = Self::layout(state);

        let [_, area] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(actions.len() as u16),
        ])
        .areas(ui.area);

        ui.clamp(area, |ui| {
            for action in actions {
                action.render(ui, state);
            }
        });
    }

    fn layout(state: &State) -> Vec<Action> {
        let mut btns = Vec::new();

        match state.mode {
            Mode::Default => {
                if !state.perms.hero_mode {
                    btns.push(Action::JoinBot);
                }

                if state.perms.can_user_upload_bots {
                    btns.push(Action::UploadBot);
                }

                if state.perms.can_user_spawn_prefabs {
                    btns.push(Action::SpawnBot);
                }
            }

            Mode::SpawningBot { .. } => {
                //
            }
        }

        btns
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug)]
enum Action {
    JoinBot,
    UploadBot,
    SpawnBot,
}

impl Action {
    fn render(self, ui: &mut Ui<Event>, state: &State) {
        match self {
            Action::JoinBot => {
                Button::new(KeyCode::Char('j'), "join-bot")
                    .throwing(Event::OpenJoinBotDialog)
                    .enabled(!state.snapshot.bots().is_empty())
                    .render(ui);
            }

            Action::UploadBot => {
                Button::new(KeyCode::Char('u'), "upload-bot")
                    .throwing(Event::OpenUploadBotDialog {
                        request: UploadBotRequest::default(),
                    })
                    .render(ui);
            }

            Action::SpawnBot => {
                Button::new(KeyCode::Char('S'), "spawn-bot")
                    .throwing(Event::OpenSpawnBotDialog)
                    .render(ui);
            }
        }
    }
}
