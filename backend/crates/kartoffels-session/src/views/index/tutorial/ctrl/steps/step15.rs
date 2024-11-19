use super::prelude::*;

static MSG: LazyLock<Msg> = LazyLock::new(|| Msg {
    title: Some(" tutorial (15/16) "),
    body: DOCS.clone(),
    buttons: vec![MsgButton::confirm("next", ())],
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
             giving your robot one point — note that you have to be _facing_ \
             the other bot in order to stab it",
        ),
        MsgLine::new(""),
        MsgLine::new("easy enough, isn't it?"),
        MsgLine::new(""),
        MsgLine::new(
            "now, to complete the tutorial, implement a bot that does a 3x3 \
             radar scan, rotates towards the closest enemy (`'@'`), goes \
             forward and stabs it; when no enemy is in sight, let your robot \
             continue moving in its current direction",
        ),
        MsgLine::new(""),
        MsgLine::new(
            "for simplicity, the enemies will not try to kill you and they \
             will be located directly north / east / west / south — i.e. you \
             don't have to worry about diagonals",
        ),
    ]
});

static MSG_RETRY: LazyLock<Msg> = LazyLock::new(|| Msg {
    title: Some(" tutorial (15/16) "),
    body: vec![MsgLine::new(
        "hmm, your robot seems to have died — delete it and upload something \
         better",
    )],
    buttons: vec![MsgButton::confirm("try-again", ())],
});

pub async fn run(ctxt: &mut TutorialCtxt) -> Result<()> {
    ctxt.game.msg(&MSG).await?;
    ctxt.game.set_help(Some(&HELP)).await?;

    loop {
        let dummies = setup_map(ctxt).await?;
        let player = ctxt.events.next_spawned_bot().await?;

        ctxt.game.set_status(Some("watching".into())).await?;

        let result = wait(ctxt, dummies, player).await?;

        ctxt.sync().await?;
        ctxt.game.set_status(None).await?;

        match result {
            ControlFlow::Continue(_) => {
                ctxt.game.msg(&MSG_RETRY).await?;
            }

            ControlFlow::Break(_) => {
                break;
            }
        }
    }

    ctxt.game.set_help(None).await?;

    Ok(())
}

async fn setup_map(ctxt: &mut TutorialCtxt) -> Result<HashSet<BotId>> {
    ctxt.delete_bots().await?;

    ctxt.world
        .set_map({
            let mut map = Map::new(uvec2(20, 10));

            map.rect(ivec2(0, 0), ivec2(19, 9), TileKind::FLOOR);
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

    let dummies = ctxt
        .world
        .create_bots(
            dummies
                .into_iter()
                .map(|pos| CreateBotRequest::new(DUMMY).at(pos).instant()),
        )
        .await?
        .into_iter()
        .collect();

    ctxt.sync().await?;
    ctxt.events.sync(ctxt.world.version()).await?;

    Ok(dummies)
}

async fn wait(
    ctxt: &mut TutorialCtxt,
    mut dummies: HashSet<BotId>,
    player: BotId,
) -> Result<ControlFlow<(), ()>> {
    loop {
        let id = ctxt.events.next_killed_bot().await?;

        if dummies.remove(&id) && dummies.is_empty() {
            return Ok(ControlFlow::Break(()));
        }

        if id == player {
            return Ok(ControlFlow::Continue(()));
        }
    }
}
