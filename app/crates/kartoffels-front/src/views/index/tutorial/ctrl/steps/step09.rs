use super::prelude::*;

static MSG: LazyLock<Msg> = LazyLock::new(|| Msg {
    title: Some("tutorial (9/16)"),

    body: vec![MsgLine::new("*time for some fun!*"), MsgLine::new("")]
        .into_iter()
        .chain(DOCS.clone())
        .collect(),

    buttons: vec![MsgButton::enter("next", ())],
});

static HELP: LazyLock<HelpMsg> = LazyLock::new(|| Msg {
    title: Some("help"),
    body: DOCS.clone(),
    buttons: vec![HelpMsgEvent::close()],
});

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
    debug!("run()");

    ctxt.game.msg(&MSG).await?;
    ctxt.game.set_help(Some(&HELP)).await?;
    ctxt.events.next_born_bot().await?;
    ctxt.sync().await?;
    ctxt.game.set_help(None).await?;
    ctxt.game.pause().await?;

    Ok(())
}
