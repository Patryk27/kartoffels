use super::prelude::*;

#[rustfmt::skip]
static DIALOG: LazyLock<Dialog<()>> = LazyLock::new(|| Dialog {
    title: Some(" tutorial "),

    body: vec![
        DialogLine::new("cool!").fg(theme::GREEN),
        DialogLine::new(""),
        DialogLine::new(
            "now let's try to unwrap what the code in `main.rs` does:",
        ),
        DialogLine::new(""),
        DialogLine::new("# motor_step()"),
        DialogLine::new(""),
        DialogLine::new(
            "this boi causes the bot to move one tile in the direction the \
             robot is currently facing (north / east / west / south)",
        ),
        DialogLine::new(""),
        DialogLine::new("# motor_turn_*()"),
        DialogLine::new(""),
        DialogLine::new(
            "this boi causes the bot to turn left (counterclockwise) or \
             right (clockwise)",
        ),
        DialogLine::new(""),
        DialogLine::new("# motor_wait()"),
        DialogLine::new(""),
        DialogLine::new(
            "this boi waits until the motor is ready to accept another command",
        ),
        DialogLine::new(""),
        DialogLine::new(
            "waiting for readiness is important, because the cpu is much \
             faster than motor, so - say - calling `motor_step()` two times in \
             a row without `motor_wait()` in-between would actually move the \
             bot just one tile forward"
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
