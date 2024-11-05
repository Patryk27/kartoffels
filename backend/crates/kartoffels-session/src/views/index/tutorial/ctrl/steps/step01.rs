use super::prelude::*;

static MSG: LazyLock<Msg<bool>> = LazyLock::new(|| Msg {
    title: Some(" tutorial (1/16) "),

    body: vec![
        MsgLine::new("hey there and welcome to kartoffels 🫡"),
        MsgLine::new(""),
        MsgLine::from_iter([
            Span::raw(
                "in just a couple of minutes we're going to make a bots' boss \
                 out of you, so buckle up and let's get started! ",
            ),
            Span::raw("*").fg(theme::RED),
        ]),
        MsgLine::new(""),
        MsgLine::new("ready?").fg(theme::GREEN).bold().centered(),
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
        MsgButton::abort("no, leave tutorial", false),
        MsgButton::confirm("yes, start tutorial", true),
    ],
});

pub async fn run(ctxt: &mut TutorialCtxt) -> Result<bool> {
    ctxt.game.show_msg(&MSG).await
}
