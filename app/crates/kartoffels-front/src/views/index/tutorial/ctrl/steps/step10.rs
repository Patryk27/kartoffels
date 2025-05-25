use super::prelude::*;

static MSG: LazyLock<Msg> = LazyLock::new(|| {
    Msg::new("tutorial (10/16)")
        .line("yes... ha ha ha... *yes*!")
        .line("")
        .line(
            "by telling the bot to always move forward instead of driving in \
             squares, we'll witness its inevitable demise by falling into the \
             void (that is, falling outside the map)",
        )
        .btn(MsgBtn::enter("next", ()))
        .build()
});

static MSG_RETRY: LazyLock<Msg> = LazyLock::new(|| {
    Msg::new("tutorial (10/16)")
        .line("hmm, your bot seems to be (still) alive")
        .line("")
        .line("i'm making a note here, not a huge success")
        .line("")
        .line(
            "make sure you removed the call to `motor_turn_right()` and \
             upload the firmware again",
        )
        .btn(MsgBtn::enter("retry", ()))
        .build()
});

static HELP_RETRY: LazyLock<HelpMsg> = LazyLock::new(|| {
    Msg::new("help")
        .line(
            "make sure you removed the call to `motor_turn_right()` and \
             upload the firmware again",
        )
        .line(MsgLine::web(""))
        .line(MsgLine::web("!! don't forget to re-run `./build` !!"))
        .btn(HelpMsgEvent::close_btn())
        .build()
});

pub async fn run(ctxt: &mut TutorialCtxt) -> Result<()> {
    info!("run()");

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
