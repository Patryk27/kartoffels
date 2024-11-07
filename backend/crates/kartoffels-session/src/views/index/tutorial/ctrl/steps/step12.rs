use super::prelude::*;

static MSG: LazyLock<Msg> = LazyLock::new(|| Msg {
    title: Some(" tutorial (12/16) "),

    body: vec![
        MsgLine::new(
            "radar returns a scan of the environment around the robot - for \
             starters, you need to know about these two functions:",
        ),
        MsgLine::new(""),
        MsgLine::new("# radar_wait()"),
        MsgLine::new(""),
        MsgLine::new(
            "similarly to `motor_wait()`, this boi waits until the radar is \
             ready to accept another command",
        ),
        MsgLine::new(""),
        MsgLine::new("# radar_scan_3x3()"),
        MsgLine::new(""),
        MsgLine::new(
            "this boi returns a scan representing the 3x3 square around your \
             bot, allowing you to see tiles and other bots:",
        ),
        MsgLine::new(""),
        MsgLine::new("    let scan = radar_scan_3x3();"),
        MsgLine::new("    let tile_in_front = scan.tile_at(0, -1);"),
        MsgLine::new("    let tile_in_back = scan.tile_at(0, 1);"),
        MsgLine::new("    let tile_to_left = scan.tile_at(-1, 0);"),
        MsgLine::new("    let tile_to_right = scan.tile_at(1, 0);"),
        MsgLine::new(""),
        MsgLine::new("    if tile_in_front == '.' {"),
        MsgLine::new("        // do something"),
        MsgLine::new("    }"),
        MsgLine::new(""),
        MsgLine::new("    if tile_to_left == '@' || tile_to_right == '@' {"),
        MsgLine::new("        // do something else"),
        MsgLine::new("    }"),
    ],

    buttons: vec![MsgButton::confirm("next", ())],
});

pub async fn run(ctxt: &mut TutorialCtxt) -> Result<()> {
    ctxt.game.show_msg(&MSG).await?;

    Ok(())
}
