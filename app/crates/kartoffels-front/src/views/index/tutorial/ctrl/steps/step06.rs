use super::prelude::*;

static MSG: LazyLock<Msg> = LazyLock::new(|| Msg {
    title: Some("tutorial (6/16)"),

    body: vec![
        MsgLine::new("nice!"),
        MsgLine::new(""),
        MsgLine::new(
            "you, _[subject name here]_ must be the pride of _[subject \
             hometown here]_",
        ),
    ],

    buttons: vec![MsgButton::enter("indeed", ())],
});

pub async fn run(ctxt: &mut TutorialCtxt) -> Result<()> {
    info!("run()");

    ctxt.game.msg(&MSG).await
}
