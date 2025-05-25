use super::prelude::*;

static MSG: LazyLock<Msg> = LazyLock::new(|| {
    Msg::new("tutorial (9/16)")
        .line("*time for some fun!*")
        .line("")
        .lines(DOCS.clone())
        .btn(MsgBtn::enter("next", ()))
        .build()
});

static HELP: LazyLock<HelpMsg> = LazyLock::new(|| Msg::help(DOCS.clone()));

static DOCS: LazyLock<Vec<MsgLine>> = LazyLock::new(|| {
    vec![
        MsgLine::new(
            "comment-out `motor_turn_right()`, so that everything the bot does \
             is just `motor_wait()` and `motor_step()`, then close this \
             message and upload the new firmware",
        ),
        MsgLine::web(""),
        MsgLine::web("!! don't forget to re-run `./build` !!"),
    ]
});

pub async fn run(ctxt: &mut TutorialCtxt) -> Result<()> {
    info!("run()");

    ctxt.game.msg(&MSG).await?;
    ctxt.game.set_help(Some(&HELP)).await?;
    ctxt.events.next_born_bot().await?;
    ctxt.sync().await?;
    ctxt.game.set_help(None).await?;
    ctxt.game.pause().await?;

    Ok(())
}
