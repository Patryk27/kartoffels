use super::prelude::*;

static DIALOG: LazyLock<Dialog<()>> = LazyLock::new(|| Dialog {
    title: Some(" tutorial "),

    body: vec![
        DialogLine::new(
            "first, the peripheral it's actually not a knife - it's *arm*",
        ),
        DialogLine::new(""),
        DialogLine::new(
            "but the only action currently exposed by this peripheral is \
             `arm_stab()`, so...",
        ),
        DialogLine::new(""),
    ]
    .into_iter()
    .chain(INSTRUCTION.clone())
    .collect(),

    buttons: vec![DialogButton::confirm(
        "scoooby doooby dooo, let's catch them",
        (),
    )],
});

static HELP: LazyLock<HelpDialog> = LazyLock::new(|| Dialog {
    title: Some(" help "),
    body: INSTRUCTION.clone(),
    buttons: vec![HelpDialogResponse::close()],
});

static INSTRUCTION: LazyLock<Vec<DialogLine>> = LazyLock::new(|| {
    vec![
        DialogLine::new("# arm_wait()"),
        DialogLine::new(""),
        DialogLine::new(
            "as you can guess, this boi waits until the arm is ready (until \
             it's _armed_, you could say)",
        ),
        DialogLine::new(""),
        DialogLine::new("# arm_stab()"),
        DialogLine::new(""),
        DialogLine::new(
            "stabs the bot that's directly in front of you, killing it and \
             giving your robot one point - note that you have to be _facing_ \
             the other bot in order to stab it",
        ),
        DialogLine::new(""),
        DialogLine::new("easy enough, isn't it?"),
        DialogLine::new(""),
        DialogLine::new(
            "now, to complete the tutorial, implement a bot that does a 3x3 \
             radar scan, rotates towards the closest enemy robot (`'@'`), goes \
             forward and stabs it; when no enemy is in sight, let your robot \
             continue moving in its current direction",
        ),
        DialogLine::new(""),
        DialogLine::new(
            "for simplicity, the enemies will not try to kill you and they \
             will be located directly north / east / west / south - i.e. you \
             don't have to worry about diagonals",
        ),
    ]
});

static DIALOG_RETRY: LazyLock<Dialog<()>> = LazyLock::new(|| Dialog {
    title: Some(" tutorial "),
    body: vec![DialogLine::new("hmm, your robot seems to have died")],
    buttons: vec![DialogButton::confirm("let's try again", ())],
});

pub async fn run(ctxt: &mut StepCtxt) -> Result<()> {
    ctxt.game.run_dialog(&DIALOG).await?;
    ctxt.game.set_help(Some(&HELP)).await?;

    loop {
        let dummies = setup_map(ctxt).await?;
        let player = ctxt.snapshots.wait_until_bot_is_spawned().await?;

        ctxt.game.set_status(Some("WATCHING".into())).await?;

        let succeeded = wait(ctxt, &dummies, player).await?;

        ctxt.wait_for_ui().await?;
        ctxt.game.set_status(None).await?;

        if succeeded {
            break;
        } else {
            ctxt.game.run_dialog(&DIALOG_RETRY).await?;
        }
    }

    ctxt.game.set_help(None).await?;

    Ok(())
}

async fn setup_map(ctxt: &mut StepCtxt) -> Result<Vec<BotId>> {
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
    ctxt: &mut StepCtxt,
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
