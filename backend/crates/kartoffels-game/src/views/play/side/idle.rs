use crate::views::play::{Event, State};
use kartoffels_ui::{Button, Ui};
use ratatui::layout::{Constraint, Layout};
use termwiz::input::KeyCode;

#[derive(Debug, Default)]
pub struct IdleSidePanel;

impl IdleSidePanel {
    pub fn render(ui: &mut Ui, state: &State) {
        let actions = Self::layout(state);

        let [_, area] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(actions.len() as u16),
        ])
        .areas(ui.area());

        ui.clamp(area, |ui| {
            for action in actions {
                action.render(ui, state);
            }
        });
    }

    fn layout(state: &State) -> Vec<Action> {
        let mut btns = Vec::new();

        if !state.perms.single_bot_mode {
            btns.push(Action::JoinBot);
        }

        btns.push(Action::UploadBot);

        if state.perms.user_can_spawn_prefabs {
            btns.push(Action::SpawnRoberto);
        }

        btns
    }
}

#[derive(Debug)]
enum Action {
    JoinBot,
    UploadBot,
    SpawnRoberto,
}

impl Action {
    fn render(self, ui: &mut Ui, state: &State) {
        match self {
            Action::JoinBot => {
                Button::new(KeyCode::Char('j'), "join bot")
                    .throwing(Event::ShowJoinBotDialog)
                    .enabled(!state.snapshot.bots().is_empty())
                    .render(ui);
            }

            Action::UploadBot => {
                Button::new(KeyCode::Char('u'), "upload bot")
                    .throwing(Event::ShowUploadBotDialog)
                    .render(ui);
            }

            Action::SpawnRoberto => {
                Button::new(KeyCode::Char('S'), "spawn roberto")
                    .throwing(Event::SpawnRoberto)
                    .render(ui);
            }
        }
    }
}
