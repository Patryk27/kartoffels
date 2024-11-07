use super::prelude::*;

static MSG: LazyLock<Msg> = LazyLock::new(|| Msg {
    title: Some(" tutorial (9/16) "),

    body: vec![MsgLine::new("*time for some fun!*"), MsgLine::new("")]
        .into_iter()
        .chain(DOCS.clone())
        .collect(),

    buttons: vec![MsgButton::confirm("next", ())],
});

static HELP: LazyLock<HelpMsg> = LazyLock::new(|| Msg {
    title: Some(" help "),
    body: DOCS.clone(),
    buttons: vec![HelpMsgResponse::close()],
});

static DOCS: LazyLock<Vec<MsgLine>> = LazyLock::new(|| {
    vec![
        MsgLine::new(
            "remove the call to `motor_turn_right()`, so that everything the \
             robot does is just `motor_wait()` and `motor_step()`, then close \
             this message and upload the updated bot",
        ),
        MsgLine::web(""),
        MsgLine::web("!! don't forget to re-run `./build` !!"),
    ]
});

pub async fn run(ctxt: &mut TutorialCtxt) -> Result<()> {
    ctxt.game.show_msg(&MSG).await?;
    ctxt.game.set_help(Some(&HELP)).await?;
    ctxt.snapshots.wait_until_bot_is_spawned().await?;
    ctxt.game.set_help(None).await?;
    ctxt.game.pause().await?;

    Ok(())
}
