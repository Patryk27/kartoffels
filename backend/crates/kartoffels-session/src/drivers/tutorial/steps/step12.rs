use super::prelude::*;

static DIALOG: LazyLock<Dialog<()>> = LazyLock::new(|| Dialog {
    title: Some(" tutorial (12/16) "),

    body: vec![
        DialogLine::new(
            "radar returns a scan of the environment around the robot - for \
             starters, you need to know about these two functions:",
        ),
        DialogLine::new(""),
        DialogLine::new("# radar_wait()"),
        DialogLine::new(""),
        DialogLine::new(
            "similarly to `motor_wait()`, this boi waits until the radar is \
             ready to accept another command",
        ),
        DialogLine::new(""),
        DialogLine::new("# radar_scan_3x3()"),
        DialogLine::new(""),
        DialogLine::new(
            "this boi returns a scan representing the 3x3 square around your \
             bot, allowing you to see tiles and other bots:",
        ),
        DialogLine::new(""),
        DialogLine::new("    let scan = radar_scan_3x3();"),
        DialogLine::new("    let tile_in_front = scan.tile_at(0, -1);"),
        DialogLine::new("    let tile_in_back = scan.tile_at(0, 1);"),
        DialogLine::new("    let tile_to_left = scan.tile_at(-1, 0);"),
        DialogLine::new("    let tile_to_right = scan.tile_at(1, 0);"),
        DialogLine::new(""),
        DialogLine::new("    if tile_in_front == '.' {"),
        DialogLine::new("        // do something"),
        DialogLine::new("    }"),
        DialogLine::new(""),
        DialogLine::new("    if tile_to_left == '@' || tile_to_right == '@' {"),
        DialogLine::new("        // do something else"),
        DialogLine::new("    }"),
    ],

    buttons: vec![DialogButton::confirm("got it", ())],
});

pub async fn run(ctxt: &mut StepCtxt) -> Result<()> {
    ctxt.game.run_dialog(&DIALOG).await?;

    Ok(())
}
