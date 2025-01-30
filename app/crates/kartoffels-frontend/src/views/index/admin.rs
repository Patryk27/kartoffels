mod login;

use crate::Background;
use anyhow::Result;
use kartoffels_store::{Session, Store};
use kartoffels_ui::{Button, Frame, KeyCode, UiWidget};
use tracing::debug;

pub async fn run(
    store: &Store,
    sess: &Session,
    frame: &mut Frame,
    bg: &Background,
) -> Result<()> {
    debug!("run()");

    if sess.with(|sess| !sess.is_admin()) {
        match login::run(store, frame, bg).await? {
            login::Event::LoggedIn => {
                sess.with(|sess| {
                    sess.make_admin();
                });
            }

            login::Event::GoBack => {
                return Ok(());
            }
        }
    }

    loop {
        let event = frame
            .update(|ui| {
                ui.widget(bg);

                ui.info_window(20, 3, Some(" admin "), |ui| {
                    Button::new(KeyCode::Char('c'), "create-world")
                        .throwing(Event::CreateWorld)
                        .centered()
                        .render(ui);

                    ui.space(1);

                    Button::new(KeyCode::Escape, "go-back")
                        .throwing(Event::GoBack)
                        .centered()
                        .render(ui);
                });
            })
            .await?;

        if let Some(event) = event {
            match event {
                Event::CreateWorld => {
                    //
                }

                Event::GoBack => {
                    return Ok(());
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Event {
    CreateWorld,
    GoBack,
}
