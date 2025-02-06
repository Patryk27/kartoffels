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
    buttons: vec![HelpMsgEvent::close()],
});

static DOCS: LazyLock<Vec<MsgLine>> = LazyLock::new(|| {
    vec![
        MsgLine::new(
            "remove the call to `motor_turn_right()`, so that everything the \
             robot does is just `motor_wait()` and `motor_step_fw()`, then \
             close this message and upload the new robot",
        ),
        MsgLine::web(""),
        MsgLine::web("!! don't forget to re-run `./build` !!"),
    ]
});

pub async fn run(ctxt: &mut TutorialCtxt) -> Result<()> {
    ctxt.game.msg(&MSG).await?;
    ctxt.game.set_help(Some(&HELP)).await?;
    ctxt.events.next_born_bot().await?;
    ctxt.sync().await?;
    ctxt.game.set_help(None).await?;
    ctxt.game.pause().await?;

    Ok(())
}
