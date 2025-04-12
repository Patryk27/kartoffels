use crate::Background;
use anyhow::{anyhow, Result};
use kartoffels_store::Store;
use kartoffels_ui::{Button, Frame, Input, KeyCode, UiWidget};
use tracing::{debug, info, warn};

pub async fn run(
    store: &Store,
    frame: &mut Frame,
    bg: &Background,
) -> Result<Event> {
    debug!("run()");

    let mut secret = Input::default().secret();

    loop {
        let event = frame
            .tick(|ui| {
                ui.add(bg);

                ui.info_window(30, 4, Some(" admin "), |ui| {
                    ui.line("enter secret:");
                    ui.add(&mut secret);
                    ui.space(1);

                    ui.row(|ui| {
                        Button::new("go-back", KeyCode::Escape)
                            .throwing(InnerEvent::GoBack)
                            .render(ui);

                        if Button::new("login", KeyCode::Enter)
                            .right_aligned()
                            .render(ui)
                            .pressed
                        {
                            ui.throw(InnerEvent::AttemptLogin(
                                secret.take_value(),
                            ));
                        }
                    });
                });
            })
            .await?;

        if let Some(event) = event {
            match event {
                InnerEvent::AttemptLogin(secret) => {
                    return if Some(secret.as_str()) == store.secret() {
                        info!("console login attempt succeeded");

                        Ok(Event::LoggedIn)
                    } else {
                        warn!(?secret, "console login attempt failed");

                        Err(anyhow!("invalid secret"))
                    };
                }

                InnerEvent::GoBack => {
                    return Ok(Event::GoBack);
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Event {
    LoggedIn,
    GoBack,
}

#[derive(Clone, Debug)]
enum InnerEvent {
    AttemptLogin(String),
    GoBack,
}
