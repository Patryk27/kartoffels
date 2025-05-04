use super::prelude::*;

static MSG: LazyLock<Msg> = LazyLock::new(|| Msg {
    title: Some(" tutorial (12/16) "),

    body: vec![
        MsgLine::new(
            "radar returns a scan of the environment around the bot, it's as \
             easy as:",
        ),
        MsgLine::new(""),
        MsgLine::new("    radar_wait(); // wait until radar is ready"),
        MsgLine::new("    radar_scan(3); // do a 3x3 scan"),
        MsgLine::new(""),
        MsgLine::new("    let front = radar_read(0, -1);"),
        MsgLine::new("    let back = radar_read(0, 1);"),
        MsgLine::new("    let left = radar_read(-1, 0);"),
        MsgLine::new("    let right = radar_read(1, 0);"),
        MsgLine::new(""),
        MsgLine::new("    if front == '.' {"),
        MsgLine::new("        // do something"),
        MsgLine::new("    }"),
        MsgLine::new(""),
        MsgLine::new("    if left == '@' || right == '@' {"),
        MsgLine::new("        // do something else"),
        MsgLine::new("    }"),
    ],

    buttons: vec![MsgButton::confirm("next", ())],
});

pub async fn run(ctxt: &mut TutorialCtxt) -> Result<()> {
    ctxt.game.msg(&MSG).await?;

    Ok(())
}
