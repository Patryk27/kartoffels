use super::prelude::*;
use futures::stream::FuturesOrdered;
use tokio_stream::StreamExt;

static MSG: LazyLock<Msg> = LazyLock::new(|| Msg {
    title: Some(" tutorial (15/16) "),
    body: DOCS.clone(),

    buttons: vec![MsgButton::confirm(
        "scoooby doooby dooo, let's catch 'em",
        (),
    )],
});

static HELP: LazyLock<HelpMsg> = LazyLock::new(|| Msg {
    title: Some(" help "),
    body: DOCS.clone(),
    buttons: vec![HelpMsgResponse::close()],
});

static DOCS: LazyLock<Vec<MsgLine>> = LazyLock::new(|| {
    vec![
        MsgLine::new("# arm_wait()"),
        MsgLine::new(""),
        MsgLine::new(
            "as you can guess, this boi waits until the arm is ready (until \
             it's _armed_, you could say)",
        ),
        MsgLine::new(""),
        MsgLine::new("# arm_stab()"),
        MsgLine::new(""),
        MsgLine::new(
            "stabs the bot that's directly in front of you, killing it and \
             giving your robot one point - note that you have to be _facing_ \
             the other bot in order to stab it",
        ),
        MsgLine::new(""),
        MsgLine::new("easy enough, isn't it?"),
        MsgLine::new(""),
        MsgLine::new(
            "now, to complete the tutorial, implement a bot that does a 3x3 \
             radar scan, rotates towards the closest enemy robot (`'@'`), goes \
             forward and stabs it; when no enemy is in sight, let your robot \
             continue moving in its current direction",
        ),
        MsgLine::new(""),
        MsgLine::new(
            "for simplicity, the enemies will not try to kill you and they \
             will be located directly north / east / west / south - i.e. you \
             don't have to worry about diagonals",
        ),
    ]
});

static MSG_RETRY: LazyLock<Msg> = LazyLock::new(|| Msg {
    title: Some(" tutorial (15/16) "),
    body: vec![MsgLine::new("hmm, your robot seems to have died")],
    buttons: vec![MsgButton::confirm("let's try again", ())],
});

pub async fn run(ctxt: &mut TutorialCtxt) -> Result<()> {
    ctxt.game.show_msg(&MSG).await?;
    ctxt.game.set_help(Some(&HELP)).await?;

    loop {
        let dummies = setup_map(ctxt).await?;
        let player = ctxt.snapshots.wait_until_bot_is_spawned().await?;

        ctxt.game.set_status(Some("watching".into())).await?;

        let succeeded = wait(ctxt, &dummies, player).await?;

        ctxt.wait_for_ui().await?;
        ctxt.game.set_status(None).await?;

        if succeeded {
            break;
        } else {
            ctxt.game.show_msg(&MSG_RETRY).await?;
        }
    }

    ctxt.game.set_help(None).await?;

    Ok(())
}

async fn setup_map(ctxt: &mut TutorialCtxt) -> Result<Vec<BotId>> {
    ctxt.destroy_bots().await?;

    ctxt.world
        .set_map({
            let mut map = Map::new(uvec2(20, 10));

            map.rect(ivec2(0, 0), ivec2(19, 9), TileBase::FLOOR);
            map
        })
        .await?;

    ctxt.world.set_spawn(ivec2(10, 9), Dir::N).await?;

    // ---

    let dummies = [
        ivec2(10, 8),
        ivec2(10, 7),
        ivec2(10, 4),
        ivec2(10, 0),
        ivec2(11, 0),
        ivec2(19, 1),
        ivec2(19, 2),
        ivec2(19, 9),
        ivec2(18, 9),
        ivec2(2, 9),
    ];

    let dummies: Vec<_> = dummies
        .into_iter()
        .map(|pos| {
            ctxt.world
                .create_bot(CreateBotRequest::new(BOT_DUMMY).at(pos))
        })
        .collect::<FuturesOrdered<_>>()
        .collect::<Result<_>>()
        .await?;

    loop {
        if ctxt
            .snapshots
            .next()
            .await?
            .bots()
            .alive()
            .has_all_of(&dummies)
        {
            break;
        }
    }

    Ok(dummies)
}

async fn wait(
    ctxt: &mut TutorialCtxt,
    dummies: &[BotId],
    player: BotId,
) -> Result<bool> {
    loop {
        let snapshot = ctxt.snapshots.next().await?;

        if !snapshot.bots().alive().has_any_of(dummies) {
            return Ok(true);
        }

        if !snapshot.bots().alive().has(player) {
            return Ok(false);
        }
    }
}
