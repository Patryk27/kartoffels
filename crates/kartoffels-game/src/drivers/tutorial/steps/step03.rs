use super::prelude::*;

const CMD: &str = "git clone -b tutorial github.com/patryk27/kartoffel";

#[rustfmt::skip]
static DIALOG: LazyLock<Dialog<&'static str>> = LazyLock::new(|| Dialog {
    title: Some(" tutorial "),

    body: vec![
        DialogLine::new("look at you, learning so fast - *NEXT LESSON!*"),
        DialogLine::new(""),
        DialogLine::new("run this:"),
        DialogLine::new(format!("    {}", CMD)),
        DialogLine::new(""),
        DialogLine::new("... and press [`enter`] once you're ready"),
    ],

    buttons: vec![
        DialogButton::new(KeyCode::Char('c'), "copy command", "copy"),
        DialogButton::confirm("i'm ready", "ready"),
    ],
});

#[allow(clippy::while_let_loop)]
pub async fn run(ctxt: &mut StepCtxt) -> Result<()> {
    loop {
        match ctxt.run_dialog(&DIALOG).await? {
            "copy" => {
                // TODO HACK
                ctxt.game
                    .open_dialog(|ui| {
                        ui.copy(CMD);
                    })
                    .await?;
            }

            _ => {
                break;
            }
        }
    }

    Ok(())
}
