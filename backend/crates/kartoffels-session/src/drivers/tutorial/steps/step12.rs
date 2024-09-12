use super::prelude::*;

#[rustfmt::skip]
static DIALOG: LazyLock<Dialog<()>> = LazyLock::new(|| Dialog {
    title: Some(" tutorial "),

    body: vec![
        DialogLine::new(
            "da radar returns a scan of the environment around the robot - for \
             starters, you need to know about these two functions:"
        ),
        DialogLine::new(""),
        DialogLine::new("# radar_wait()"),
        DialogLine::new(""),
        DialogLine::new(
            "similarly to `motor_wait()`, this boi waits until the radar is \
             ready to accept another command; it's important to call this \
             function before operating on the radar",
        ),
        DialogLine::new(""),
        DialogLine::new("# radar_scan_3x3()"),
        DialogLine::new(""),
        DialogLine::new(
            "this boi returns a _yx-indexed_ array representing tiles in a \
             3x3 square around the robot, so basically:",
        ),
        DialogLine::new(""),
        DialogLine::new("    let scan = radar_scan_3x3();"),
        DialogLine::new(""),
        DialogLine::new("... yields:"),
        DialogLine::new(""),
        DialogLine::new("\t- `scan[1][1]` = center, always `'@'`"),
        DialogLine::new("\t- `scan[0][1]` = tile in front of us"),
        DialogLine::new("\t- `scan[2][1]` = tile behind us"),
        DialogLine::new("\t- `scan[1][0]` = tile to our left"),
        DialogLine::new("\t- `scan[1][2]` = tile to our right"),
        DialogLine::new(""),
        DialogLine::new(
            "the tiles correspond to what you see on the map, so there's \
             `'@'`, `'.'` and `' '`",
        ),
    ],

    buttons: vec![
        DialogButton::confirm("got it", ()),
    ],
});

pub async fn run(ctxt: &mut StepCtxt) -> Result<()> {
    ctxt.run_dialog(&DIALOG).await?;

    Ok(())
}
