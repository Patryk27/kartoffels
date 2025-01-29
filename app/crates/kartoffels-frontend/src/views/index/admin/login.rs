use crate::Background;
use anyhow::{anyhow, Result};
use kartoffels_store::Store;
use kartoffels_ui::{Button, Input, KeyCode, Term, UiWidget};
use tracing::{debug, info, warn};

pub async fn run(
    store: &Store,
    term: &mut Term,
    bg: &Background,
) -> Result<Event> {
    debug!("run()");

    if store.secret().is_none() {
        return Ok(Event::GoBack);
    }

    let mut secret = Input::default().secret();
    let mut tries = 0;

    loop {
        let event = term
            .frame(|ui| {
                ui.widget(bg);

                ui.info_window(30, 4, Some(" admin "), |ui| {
                    ui.line("enter secret:");
                    ui.widget(&mut secret);
                    ui.space(1);

                    ui.row(|ui| {
                        Button::new(KeyCode::Escape, "go-back")
                            .throwing(InnerEvent::GoBack)
                            .render(ui);

                        if Button::new(KeyCode::Enter, "login")
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
                    info!(?secret, "admin login attempt");

                    if Some(secret.as_str()) == store.secret() {
                        return Ok(Event::LoggedIn);
                    } else {
                        warn!(?secret, ?tries, "admin login attempt failed");
                    }

                    tries += 1;

                    if tries >= 3 {
                        return Err(anyhow!("too many login attempts"));
                    }
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
