use super::prelude::*;

static MSG: LazyLock<Msg> = LazyLock::new(|| {
    Msg::new("tutorial (8/16)")
        .line(MsgLine::new("cool!").fg(theme::GREEN))
        .line("")
        .line("now let's try to unpack what the code does:")
        .line("")
        .line("# motor_step()")
        .line("")
        .line(
            "this boi causes the bot to move one tile forward in the direction \
             it's facing (north / east / west / south)",
        )
        .line("")
        .line("# motor_turn_*()")
        .line("")
        .line(
            "this boi causes the bot to turn left (counterclockwise) or right \
             (clockwise)",
        )
        .line("")
        .line("# motor_wait()")
        .line("")
        .line(
            "this boi waits until the motor is ready to accept another command",
        )
        .line("")
        .line(
            "waiting for readiness is important, because the cpu is much \
             faster than motor, so calling `motor_step()` two times in a \
             row without `motor_wait()` in-between would actually move you \
             just one tile forward",
        )
        .btn(MsgBtn::enter("next", ()))
        .build()
});

pub async fn run(ctxt: &mut TutorialCtxt) -> Result<()> {
    info!("run()");

    ctxt.game.msg(&MSG).await
}
