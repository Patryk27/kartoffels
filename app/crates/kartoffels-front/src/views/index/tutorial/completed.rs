use crate::{
    theme, Clear, FadeCtrl, FadeCtrlEvent, Frame, Msg, MsgButton, MsgLine,
};
use anyhow::Result;
use ratatui::style::Stylize;
use std::sync::LazyLock;

static MSG: LazyLock<Msg<Event>> = LazyLock::new(|| Msg {
    title: Some(" tutorial "),

    body: vec![
        MsgLine::new("🥔✨ *yay, you made it!* ✨🥔")
            .fg(theme::GREEN)
            .bold()
            .centered(),
        MsgLine::new(""),
        MsgLine::new("not sure if mom and dad are proud of you, but i am !!"),
        MsgLine::new(""),
        MsgLine::new(
            "kartoffels is all about discovery, so we won't go through the \
             remaining functions - from now on you're on your own",
        ),
        MsgLine::new(""),
        MsgLine::new(
            "be gay, do crime, have fun, and remember the power of potato!",
        ),
    ],

    buttons: vec![MsgButton::confirm("complete", Event::Complete)],
});

pub async fn run(frame: &mut Frame) -> Result<()> {
    let mut fade = FadeCtrl::default().fade_in(true);

    loop {
        let event = frame
            .tick(|ui| {
                fade.render(ui, |ui| {
                    ui.add(Clear);
                    ui.add(&*MSG);
                });
            })
            .await?;

        if event.is_some() {
            return Ok(());
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Event {
    Complete,
}

impl FadeCtrlEvent for Event {
    fn needs_fade_out(&self) -> bool {
        true
    }
}
