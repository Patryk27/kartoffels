use super::prelude::*;

static DIALOG: LazyLock<Dialog<()>> = LazyLock::new(|| Dialog {
    title: Some(" tutorial "),

    body: vec![
        DialogLine::new(
            "anyway, close this message to resume the game and let's see the \
             robot in action",
        ),
        DialogLine::new(""),
        DialogLine::new(
            "if everything goes correctly, we should see the robot driving in \
             squares, *how exquisite*!",
        ),
    ],

    buttons: vec![DialogButton::confirm(
        "yes, let's see the robot driving in squares",
        (),
    )],
});

pub async fn run(ctxt: &mut StepCtxt) -> Result<()> {
    ctxt.game.run_dialog(&DIALOG).await?;
    ctxt.game.resume().await?;

    time::sleep(Duration::from_secs(6)).await;

    ctxt.destroy_bots().await?;

    Ok(())
}
