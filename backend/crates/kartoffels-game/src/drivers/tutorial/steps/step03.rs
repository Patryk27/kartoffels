use super::prelude::*;

const CMD: &str = "git clone github.com/patryk27/kartoffel";

#[rustfmt::skip]
static DIALOG: LazyLock<Dialog<&'static str>> = LazyLock::new(|| Dialog {
    title: Some(" tutorial "),

    body: vec![
        DialogLine::new("look at you, learning so fast - *NEXT LESSON!*"),
        DialogLine::new(""),
        DialogLine::new("run this:"),
        DialogLine::new(format!("    {CMD}")),
        DialogLine::new(""),
        DialogLine::new("... and press [`enter`] once you're ready"),
    ],

    buttons: vec![
        DialogButton::new(KeyCode::Char('c'), "copy command", "copy"),
        DialogButton::confirm("i'm ready", "ready"),
    ],
});

pub async fn run(ctxt: &mut StepCtxt) -> Result<()> {
    loop {
        match ctxt.run_dialog(&DIALOG).await? {
            "copy" => {
                ctxt.game.copy_to_clipboard(CMD).await?;
            }

            "ready" => {
                break;
            }

            _ => unreachable!(),
        }
    }

    Ok(())
}
