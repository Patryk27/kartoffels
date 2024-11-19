use super::prelude::*;

static MSG: LazyLock<Msg> = LazyLock::new(|| Msg {
    title: Some(" tutorial (8/16) "),

    body: vec![
        MsgLine::new("cool!").fg(theme::GREEN),
        MsgLine::new(""),
        MsgLine::new(
            "now let's try to unwrap what the code in `main.rs` does:",
        ),
        MsgLine::new(""),
        MsgLine::new("# motor_step()"),
        MsgLine::new(""),
        MsgLine::new(
            "this boi causes the bot to move one tile in the direction the \
             robot is currently facing (north / east / west / south)",
        ),
        MsgLine::new(""),
        MsgLine::new("# motor_turn_*()"),
        MsgLine::new(""),
        MsgLine::new(
            "this boi causes the bot to turn left (counterclockwise) or \
             right (clockwise)",
        ),
        MsgLine::new(""),
        MsgLine::new("# motor_wait()"),
        MsgLine::new(""),
        MsgLine::new(
            "this boi waits until the motor is ready to accept another command",
        ),
        MsgLine::new(""),
        MsgLine::new(
            "waiting for readiness is important, because the cpu is much \
             faster than motor, so calling `motor_step()` two times in a row \
             without `motor_wait()` in-between would actually move the bot \
             just one tile forward",
        ),
    ],

    buttons: vec![MsgButton::confirm("next", ())],
});

pub async fn run(ctxt: &mut TutorialCtxt) -> Result<()> {
    ctxt.game.msg(&MSG).await?;

    Ok(())
}
