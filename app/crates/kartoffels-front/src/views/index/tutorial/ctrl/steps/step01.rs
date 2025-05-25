use super::prelude::*;

static MSG: LazyLock<Msg<bool>> = LazyLock::new(|| {
    Msg::new("tutorial")
        .line("hey there and welcome to kartoffels!")
        .line("")
        .line(
            "this is a game when you're given a potato and your job is to \
             implement a firmware for it -- it's beginner friendly and \
             open-ended, and you get to set your own rules (sorta)",
        )
        .line("")
        .line(
            "this tutorial covers all of the basic stuff, from user interface \
             to how the bots are programmed - you'll need git, your favorite \
             text editor, and a couple of minutes",
        )
        .line("")
        .line(
            MsgLine::from_iter([
                Span::raw("* ").fg(theme::RED),
                Span::raw(
                    "kartoffels ltd is not responsible for loss of hearing, \
                     loss of sight, sudden feeling of the flight and fight \
                     syndrome, wanting to do origami but being unable to etc.",
                ),
            ])
            .fg(theme::GRAY)
            .right_aligned(),
        )
        .btn(MsgBtn::escape("exit", false))
        .btn(MsgBtn::enter("start", true))
        .build()
});

pub async fn run(ctxt: &mut TutorialCtxt) -> Result<bool> {
    info!("run()");

    let msg = ctxt.game.msg_ex(&MSG).await?;

    if *msg.answer() {
        msg.close().await?;

        Ok(true)
    } else {
        Ok(false)
    }
}
