use super::prelude::*;

#[rustfmt::skip]
static DIALOG: LazyLock<Dialog<'static, Response>> = LazyLock::new(|| Dialog {
    title: Some(" tutorial "),

    body: vec![
        DialogLine::raw("hey there and welcome to kartoffels 🫡"),
        DialogLine::raw(""),
        DialogLine::from_iter([
            Span::raw(
                "in a couple of minutes we'll make a bots' boss out of you, so \
                 let's get down to business!"
            ),
            Span::raw("*").fg(theme::RED),
        ]),
        DialogLine::raw(""),
        DialogLine::raw("lesson #1:").bold(),
        DialogLine::raw("you can navigate the interface using keyboard and/or mouse"),
        DialogLine::raw("(that includes when you're connected through the terminal)"),
        DialogLine::raw(""),
        DialogLine::raw("lesson #2:").bold(),
        DialogLine::raw("pressing Ctrl-c will always bring you to the main menu"),
        DialogLine::raw(""),
        DialogLine::from_iter([
            Span::raw("* ").fg(theme::RED),
            Span::raw(
                "kartoffels ltd is not responsible for loss of hearing, loss \
                 of sight, sudden feeling of the flight and fight syndrome, \
                 wanting to do origami but being unable to etc."
            ),
        ]).fg(theme::DARK_GRAY).right_aligned(),
    ],

    buttons: vec![
        DialogButton::abort("leave tutorial", Response::Abort),
        DialogButton::confirm("got it", Response::Confirm),
    ],
});

#[derive(Clone, Copy, Debug)]
pub enum Response {
    Abort,
    Confirm,
}

pub async fn run(ctxt: &mut StepCtxt<'_>) -> Result<Response> {
    ctxt.game
        .set_policy(Policy {
            ui_enabled: false,
            user_can_pause_world: false,
            user_can_configure_world: false,
            user_can_manage_bots: false,
            pause_is_propagated: true,
        })
        .await?;

    ctxt.dialog(&DIALOG).await
}
