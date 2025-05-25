use super::prelude::*;

static MSG: LazyLock<Msg<bool>> = LazyLock::new(|| {
    Msg::new("tutorial (5/16)")
        .line(
            "as you can see, the code in `main.rs` just calls a couple of \
             functions in a loop - let's see them in action!",
        )
        .line("")
        .lines(DOCS.clone())
        .btn(MsgBtn::escape("back", false))
        .btn(MsgBtn::enter("next", true))
        .build()
});

static HELP: LazyLock<HelpMsg> = LazyLock::new(|| Msg::help(DOCS.clone()));

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

pub async fn run(ctxt: &mut TutorialCtxt) -> Result<bool> {
    info!("run()");

    if !ctxt.game.msg(&MSG).await? {
        return Ok(false);
    }

    ctxt.game.set_help(Some(&HELP)).await?;
    ctxt.events.next_born_bot().await?;
    ctxt.sync().await?;
    ctxt.game.set_help(None).await?;
    ctxt.game.pause().await?;

    Ok(true)
}
