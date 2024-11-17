use super::prelude::*;

static MSG: LazyLock<Msg> = LazyLock::new(|| Msg {
    title: Some(" tutorial (5/16) "),

    body: vec![
        MsgLine::new(
            "as you can see, the code in `main.rs` just calls a couple of \
             functions in a loop - let's see them in action!",
        ),
        MsgLine::new(""),
    ]
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
        MsgLine::new("run this:"),
        MsgLine::web("    ./build"),
        MsgLine::ssh("    ./build --copy"),
        MsgLine::new(""),
        MsgLine::new(
            "... then close this message and press [`u`] to upload the bot",
        ),
        MsgLine::web(""),
        MsgLine::web(
            "when the file picker opens, choose a file called `kartoffel` - it \
             should be located next to `README.md` etc.",
        ),
    ]
});

pub async fn run(ctxt: &mut TutorialCtxt) -> Result<()> {
    ctxt.game.msg(&MSG).await?;
    ctxt.game.set_help(Some(&HELP)).await?;
    ctxt.snapshots.next_uploaded_bot().await?;
    ctxt.sync().await?;
    ctxt.game.set_help(None).await?;
    ctxt.game.pause().await?;

    Ok(())
}
