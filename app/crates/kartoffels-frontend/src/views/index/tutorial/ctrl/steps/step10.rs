use super::prelude::*;

static MSG: LazyLock<Msg> = LazyLock::new(|| Msg {
    title: Some(" tutorial (10/16) "),

    body: vec![
        MsgLine::new("yes... ha ha ha... *YES*!"),
        MsgLine::new(""),
        MsgLine::new(
            "by telling the robot to always move forward instead of driving in \
             squares, we should see the robot, well, moving forward and \
             unknowingly falling out of the map",
        ),
    ],

    buttons: vec![MsgButton::confirm("next", ())],
});

static MSG_RETRY: LazyLock<Msg> = LazyLock::new(|| Msg {
    title: Some(" tutorial (10/16) "),

    body: vec![
        MsgLine::new("hmm, your robot seems to be still alive"),
        MsgLine::new(""),
        MsgLine::new(
            "this wasn't a triumph, i'm making a note here, NOT a huge \
             success",
        ),
        MsgLine::new(""),
        MsgLine::new(
            "make sure you removed the call to `motor_turn_right()` and upload \
             the bot again",
        ),
    ],

    buttons: vec![MsgButton::confirm("try-again", ())],
});

static HELP_RETRY: LazyLock<HelpMsg> = LazyLock::new(|| Msg {
    title: Some(" help "),

    body: vec![
        MsgLine::new(
            "make sure you removed the call to `motor_turn_right()` and upload \
             the bot again",
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
        ctxt.game.set_status(Some("watching".into())).await?;

        let result =
            time::timeout(Duration::from_secs(10), ctxt.events.next_died_bot())
                .await;

        ctxt.delete_bots().await?;
        ctxt.game.set_status(None).await?;

        match result {
            Ok(result) => {
                return result.map(drop);
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
