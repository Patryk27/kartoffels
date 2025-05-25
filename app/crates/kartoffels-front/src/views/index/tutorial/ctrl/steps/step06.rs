use super::prelude::*;

static MSG: LazyLock<Msg> = LazyLock::new(|| {
    Msg::new("tutorial (6/16)")
        .line("nice!")
        .line("")
        .line(
            "you, _[subject name here]_ must be the pride of _[subject \
             hometown here]_",
        )
        .btn(MsgBtn::enter("indeed", ()))
        .build()
});

pub async fn run(ctxt: &mut TutorialCtxt) -> Result<()> {
    info!("run()");

    ctxt.game.msg(&MSG).await
}
