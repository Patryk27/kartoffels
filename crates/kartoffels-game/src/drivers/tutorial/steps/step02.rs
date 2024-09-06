use super::prelude::*;

#[rustfmt::skip]
static DIALOG: LazyLock<Dialog<'static, ()>> = LazyLock::new(|| Dialog {
    title: Some(" tutorial "),

    body: vec![
        DialogLine::raw("wow, you're learning so fast"),
        DialogLine::raw(""),
        DialogLine::from_iter([
            Span::raw("NEXT LESSON!").bold(),
            Span::raw(" -- run this:"),
        ]),
        DialogLine::raw(""),
        DialogLine::raw("    git clone -b tutorial github.com/patryk27/kartoffel")
            .fg(theme::WASHED_PINK),
        DialogLine::raw(""),
        DialogLine::raw("... and press enter once you're ready"),
    ],

    buttons: vec![
        DialogButton::confirm("i'm ready", ()),
    ],
});

pub async fn run(ctxt: &mut StepCtxt<'_>) -> Result<()> {
    ctxt.dialog(&DIALOG).await?;

    Ok(())
}
