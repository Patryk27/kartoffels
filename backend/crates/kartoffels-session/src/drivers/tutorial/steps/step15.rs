use super::prelude::*;

#[rustfmt::skip]
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

    buttons: vec![
        DialogButton::confirm("scoooby doooby dooo, let's catch them", ()),
    ],
});

#[rustfmt::skip]
static HELP: LazyLock<HelpDialog> = LazyLock::new(|| Dialog {
    title: Some(" help "),
    body: INSTRUCTION.clone(),

    buttons: vec![
        DialogButton::confirm("got it", HelpDialogResponse::Close),
    ],
});

#[rustfmt::skip]
static INSTRUCTION: LazyLock<Vec<DialogLine>> = LazyLock::new(|| vec![
    DialogLine::new("# arm_wait()"),
    DialogLine::new(""),
    DialogLine::new(
        "as you can guess, this boi waits until the arm is ready (until it's \
         armed, you could say)",
    ),
    DialogLine::new(""),
    DialogLine::new("# arm_stab()"),
    DialogLine::new(""),
    DialogLine::new(
        "stabs the bot that's directly in front of you, killing it and giving \
         your robot one point - note that you have to be _facing_ the other \
         bot in order to stab it",
    ),
    DialogLine::new(""),
    DialogLine::new("easy enough, isn't it?"),
    DialogLine::new(""),
    DialogLine::new(
        "now, to complete the tutorial, implement a bot that does a 3x3 radar \
         scan, rotates towards the closest enemy robot (`'@'`), goes forward \
         and stabs it; when no enemy is in sight, let your robot continue \
         moving in its current direction",
    ),
    DialogLine::new(""),
    DialogLine::new(
        "for simplicity, the enemies will not try to kill you and they will be \
         located directly north / east / west / south - i.e. you don't have to \
         worry about diagonals",
    ),
]);

#[rustfmt::skip]
static DIALOG_RETRY: LazyLock<Dialog<()>> = LazyLock::new(|| Dialog {
    title: Some(" tutorial "),

    body: vec![
        DialogLine::new("hmm, your robot seems to have died"),
    ],

    buttons: vec![
        DialogButton::confirm("let's try again", ()),
    ],
});

pub async fn run(ctxt: &mut StepCtxt) -> Result<()> {
    ctxt.run_dialog(&DIALOG).await?;
    ctxt.game.set_help(Some(&HELP)).await?;

    loop {
        setup_map(ctxt).await?;
        setup_dummies(ctxt).await?;

        let user_bot_id = ctxt.wait_until_bot_is_spawned().await?;

        ctxt.game.set_status(Some("WATCHING".into())).await?;

        let outcome = wait(ctxt, user_bot_id).await?;

        ctxt.game.set_status(None).await?;

        match outcome {
            Ok(()) => {
                break;
            }

            Err(()) => {
                ctxt.run_dialog(&DIALOG_RETRY).await?;
            }
        }
    }

    ctxt.game.set_help(None).await?;

    Ok(())
}

async fn setup_map(ctxt: &mut StepCtxt) -> Result<()> {
    ctxt.destroy_bots().await?;

    ctxt.world
        .set_map({
            let mut map = Map::new(uvec2(20, 10));

            map.rect(ivec2(0, 0), ivec2(19, 9), Tile::new(TileBase::FLOOR));
            map
        })
        .await?;

    ctxt.world
        .set_spawn(Some(ivec2(10, 9)), Some(Dir::Up))
        .await?;

    Ok(())
}

async fn setup_dummies(ctxt: &mut StepCtxt) -> Result<()> {
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
        ivec2(0, 9),
    ];

    for pos in dummies {
        ctxt.world.create_bot(BOT_DUMMY, Some(pos)).await?;
    }

    Ok(())
}

async fn wait(
    ctxt: &mut StepCtxt,
    user_bot_id: BotId,
) -> Result<Result<(), ()>> {
    ctxt.game
        .poll(move |ctxt| {
            let bots = ctxt.world.bots().alive();

            if bots.by_id(user_bot_id).is_none() {
                return Poll::Ready(Err(()));
            }

            if bots.len() == 1 {
                return Poll::Ready(Ok(()));
            }

            Poll::Pending
        })
        .await
}
