use super::prelude::*;

static DIALOG: LazyLock<Dialog<bool>> = LazyLock::new(|| Dialog {
    title: Some(" tutorial (1/16) "),

    body: vec![
        DialogLine::new("hey there and welcome to kartoffels 🫡"),
        DialogLine::new(""),
        DialogLine::from_iter([
            Span::raw(
                "in just a couple of minutes we're going to make a bots' boss \
                 out of you, so buckle up and let's get started! ",
            ),
            Span::raw("*").fg(theme::RED),
        ]),
        DialogLine::new(""),
        DialogLine::new("ready?").fg(theme::GREEN).bold().centered(),
        DialogLine::new(""),
        DialogLine::from_iter([
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
        DialogButton::abort("no, leave tutorial", false),
        DialogButton::confirm("yes, start tutorial", true),
    ],
});

pub async fn run(ctxt: &mut StepCtxt) -> Result<bool> {
    ctxt.game.run_dialog(&DIALOG).await
}
