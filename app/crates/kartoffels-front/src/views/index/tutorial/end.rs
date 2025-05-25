use crate::{Clear, Fade, Frame, Msg, MsgBtn, MsgLine, theme};
use anyhow::Result;
use kartoffels_store::Store;
use ratatui::style::Stylize;
use std::sync::LazyLock;

static MSG: LazyLock<Msg> = LazyLock::new(|| {
    Msg::new("tutorial")
        .line(
            MsgLine::new("*yay, you made it!*")
                .fg(theme::GREEN)
                .centered(),
        )
        .line("")
        .line("not sure if mom and dad are proud of you, but i am !!")
        .line("")
        .line(
            "kartoffels is all about discovery, so i'll be leaving you to \
             your own devices now - snoop around the user interface, snoop \
             around the api, be gay, do crime, have fun, and:",
        )
        .line("")
        .line("remember the power of potato!")
        .btn(MsgBtn::enter("complete", ()))
        .build()
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
