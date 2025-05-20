use super::prelude::*;

static MSG: LazyLock<Msg> = LazyLock::new(|| Msg {
    title: Some(" tutorial (15/16) "),
    body: DOCS.clone(),
    buttons: vec![MsgButton::confirm("next", ())],
});

static HELP: LazyLock<HelpMsg> = LazyLock::new(|| Msg {
    title: Some(" help "),
    body: DOCS.clone(),
    buttons: vec![HelpMsgEvent::close()],
});

static DOCS: LazyLock<Vec<MsgLine>> = LazyLock::new(|| {
    vec![
        MsgLine::new("# arm_wait()"),
        MsgLine::new(""),
        MsgLine::new(
            "as you can guess, this boi waits until the arm is ready (until \
             it's _armed_ you could say, heh)",
        ),
        MsgLine::new(""),
        MsgLine::new("# arm_stab()"),
        MsgLine::new(""),
        MsgLine::new(
            "stabs the bot that's directly in front of you, killing it and \
             giving you one point — note that you have to be _facing_ the \
             other machine in order to stab it",
        ),
        MsgLine::new(""),
        MsgLine::new("easy enough, isn't it?"),
        MsgLine::new(""),
        MsgLine::new(
            "now, to complete the tutorial, implement a bot that performs a \
             3x3 scan, rotates towards the closest enemy (`'@'`), drives \
             forward and stabs it; when no enemy is in sight, let your bot \
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
        "hmm, your bot seems to have died — delete it and upload something \
         better, i know you have it in you",
    )],
    buttons: vec![MsgButton::confirm("try-again", ())],
});

pub async fn run(ctxt: &mut TutorialCtxt) -> Result<()> {
    ctxt.game.msg(&MSG).await?;
    ctxt.game.set_help(Some(&HELP)).await?;

    loop {
        let dummies = setup_map(ctxt).await?;
        let player = ctxt.events.next_born_bot().await?;

        ctxt.game.set_label(Some("watching".into())).await?;

        let result = wait(ctxt, dummies, player).await?;

        ctxt.sync().await?;
        ctxt.game.set_label(None).await?;

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

async fn setup_map(ctxt: &mut TutorialCtxt) -> Result<HashSet<w::BotId>> {
    ctxt.delete_bots().await?;

    ctxt.world
        .set_map(w::Map::new(uvec2(20, 10)).filled_with(w::TileKind::FLOOR))
        .await?;

    ctxt.world.set_spawn(ivec2(10, 9), w::AbsDir::N).await?;

    // ---

    const DUMMIES: &[IVec2] = &[
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

    let mut dummies = HashSet::new();

    for &pos in DUMMIES {
        dummies.insert(
            ctxt.world
                .create_bot(w::CreateBotRequest::new(DUMMY).at(pos).instant())
                .await?,
        );
    }

    ctxt.sync().await?;
    ctxt.events.sync(ctxt.world.version()).await?;

    Ok(dummies)
}

async fn wait(
    ctxt: &mut TutorialCtxt,
    mut dummies: HashSet<w::BotId>,
    player: w::BotId,
) -> Result<ControlFlow<(), ()>> {
    loop {
        let id = ctxt.events.next_died_bot().await?;

        if dummies.remove(&id) && dummies.is_empty() {
            return Ok(ControlFlow::Break(()));
        }

        if id == player {
            return Ok(ControlFlow::Continue(()));
        }
    }
}
