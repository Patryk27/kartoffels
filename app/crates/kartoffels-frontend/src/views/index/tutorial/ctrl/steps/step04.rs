use super::prelude::*;

static MSG: LazyLock<Msg> = LazyLock::new(|| Msg {
    title: Some(" tutorial (4/16) "),

    body: vec![
        MsgLine::new("fabulous!").fg(theme::PINK).bold(),
        MsgLine::new(""),
        MsgLine::new(
            "now launch vscode, vim, emacs or whatever gives your life colors \
             and open `main.rs` from the cloned repository",
        ),
        MsgLine::new(""),
        MsgLine::new(
            "for, you see, writing a bot is similar to writing a regular rust \
             program — but it's also different, _mucho_ different",
        ),
        MsgLine::new(""),
        MsgLine::new(
            "like, you-dont-have-access-to-standard-library different",
        ),
        MsgLine::new(""),
        MsgLine::new(
            "there's no `std::fs`, no `std::net` - everything your bot has \
             access to is a bit of memory, motor, radar, and serial port",
        ),
        MsgLine::new(""),
        MsgLine::new("you know, like the people in ancient rome did"),
    ],

    buttons: vec![MsgButton::confirm("next", ())],
});

pub async fn run(ctxt: &mut TutorialCtxt) -> Result<()> {
    ctxt.game.msg(&MSG).await?;

    Ok(())
}
