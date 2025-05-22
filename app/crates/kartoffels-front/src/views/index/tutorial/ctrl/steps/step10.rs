use super::prelude::*;

static MSG: LazyLock<Msg> = LazyLock::new(|| Msg {
    title: Some("tutorial (10/16)"),

    body: vec![
        MsgLine::new("yes... ha ha ha... *yes*!"),
        MsgLine::new(""),
        MsgLine::new(
            "by telling the bot to always move forward instead of driving in \
             squares, we'll witness its inevitable demise by falling into the \
             void (that is, falling outside the map)",
        ),
    ],

    buttons: vec![MsgButton::enter("next", ())],
});

static MSG_RETRY: LazyLock<Msg> = LazyLock::new(|| Msg {
    title: Some("tutorial (10/16)"),

    body: vec![
        MsgLine::new("hmm, your bot seems to be (still) alive"),
        MsgLine::new(""),
        MsgLine::new("i'm making a note here, not a huge success"),
        MsgLine::new(""),
        MsgLine::new(
            "make sure you removed the call to `motor_turn_right()` and \
             upload the firmware again",
        ),
    ],

    buttons: vec![MsgButton::enter("try-again", ())],
});

static HELP_RETRY: LazyLock<HelpMsg> = LazyLock::new(|| Msg {
    title: Some("help"),

    body: vec![
        MsgLine::new(
            "make sure you removed the call to `motor_turn_right()` and \
             upload the firmware again",
        ),
        MsgLine::web(""),
        MsgLine::web("!! don't forget to re-run `./build` !!"),
    ],

    buttons: vec![HelpMsgEvent::close()],
});

pub async fn run(ctxt: &mut TutorialCtxt) -> Result<()> {
    ctxt.game.msg(&MSG).await?;
    ctxt.game.resume().await?;

    loop {
        ctxt.game.set_label(Some("watching".into())).await?;

        let result = time::timeout(Duration::from_secs(10), async {
            ctxt.events.next_died_bot().await?;

            Ok(())
        })
        .await;

        ctxt.delete_bots().await?;
        ctxt.game.set_label(None).await?;

        match result {
            Ok(result) => {
                return result;
            }

            Err(_) => {
                ctxt.game.msg(&MSG_RETRY).await?;
                ctxt.game.set_help(Some(&HELP_RETRY)).await?;
                ctxt.events.next_born_bot().await?;
                ctxt.game.set_help(None).await?;
            }
        }
    }
}
