use super::prelude::*;

static MSG: LazyLock<Msg<bool>> = LazyLock::new(|| Msg {
    title: Some("tutorial"),

    body: vec![
        MsgLine::new("hey there and welcome to kartoffels!"),
        MsgLine::new(""),
        MsgLine::new(
            "this is a game when you're given a potato and your job is to \
             implement a firmware for it -- it's beginner friendly and \
             open-ended, and you get to set your own rules (sorta)",
        ),
        MsgLine::new(""),
        MsgLine::new(
            "this tutorial covers all of the basic stuff, from user interface \
             to how the bots are programmed - you'll need git, your favorite \
             text editor, and a couple of minutes",
        ),
        MsgLine::new(""),
        MsgLine::from_iter([
            Span::raw("* ").fg(theme::RED),
            Span::raw(
                "kartoffels ltd is not responsible for loss of hearing, loss \
                 of sight, sudden feeling of the flight and fight syndrome, \
                 wanting to do origami but being unable to etc.",
            ),
        ])
        .fg(theme::DARK_GRAY)
        .right_aligned(),
    ],

    buttons: vec![
        MsgButton::escape("exit", false),
        MsgButton::enter("start", true),
    ],
});

pub async fn run(ctxt: &mut TutorialCtxt) -> Result<bool> {
    debug!("run()");

    ctxt.game.msg(&MSG).await
}
