use anyhow::Result;
use kartoffels_ui::{
    theme, Clear, Fade, FadeDir, Msg, MsgButton, MsgLine, Term, UiWidget,
};
use ratatui::style::Stylize;
use std::sync::LazyLock;

static MSG: LazyLock<Msg> = LazyLock::new(|| Msg {
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
            "be gay, do crime, have fun and remember the power of potato!",
        ),
    ],

    buttons: vec![MsgButton::confirm("complete", ())],
});

pub async fn run(term: &mut Term) -> Result<()> {
    let mut fade_in = Some(Fade::new(FadeDir::In));
    let mut fade_out: Option<Fade> = None;

    loop {
        let event = term
            .frame(|ui| {
                Clear::render(ui);
                MSG.render(ui);

                if let Some(fade) = &fade_in {
                    if fade.render(ui).is_completed() {
                        fade_in = None;
                    }
                }

                if let Some(fade) = &fade_out {
                    fade.render(ui);
                }
            })
            .await?;

        if let Some(fade) = &fade_out {
            if fade.is_completed() {
                return Ok(());
            }

            continue;
        }

        if event.is_some() {
            fade_out = Some(Fade::new(FadeDir::Out));
        }
    }
}
