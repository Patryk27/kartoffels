use crate::{Clear, Fade, Frame, Msg, MsgButton, MsgLine, theme};
use anyhow::Result;
use kartoffels_store::Store;
use ratatui::style::Stylize;
use std::sync::LazyLock;

static MSG: LazyLock<Msg> = LazyLock::new(|| Msg {
    title: Some("tutorial"),

    body: vec![
        MsgLine::new("*yay, you made it!*")
            .fg(theme::GREEN)
            .bold()
            .centered(),
        MsgLine::new(""),
        MsgLine::new("not sure if mom and dad are proud of you, but i am !!"),
        MsgLine::new(""),
        MsgLine::new(
            "kartoffels is all about discovery, so i'll be leaving you to \
             your own devices now - snoop around the user interface, snoop \
             around the api, be gay, do crime, have fun, and:",
        ),
        MsgLine::new(""),
        MsgLine::new("remember the power of potato!"),
    ],

    buttons: vec![MsgButton::enter("complete", ())],
});

pub async fn run(store: &Store, frame: &mut Frame) -> Result<()> {
    let mut fade = Fade::new(store, true);

    loop {
        let event = frame
            .render(|ui| {
                ui.add(Clear);
                ui.add(&*MSG);
                fade.render(ui);
            })
            .await?;

        if event.is_some() {
            fade.out(());
        }

        if fade.poll().is_some() {
            return Ok(());
        }
    }
}
