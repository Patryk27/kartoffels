use super::prelude::*;

static DIALOG: LazyLock<Dialog<()>> = LazyLock::new(|| Dialog {
    title: Some(" tutorial "),

    body: vec![
        DialogLine::new(
            "how about we implement a *line following robot* to solidify all \
             this knowledge, eh?",
        ),
        DialogLine::new(""),
    ]
    .into_iter()
    .chain(INSTRUCTION.clone())
    .collect(),

    buttons: vec![DialogButton::confirm("let's implement a line-follower", ())],
});

static HELP: LazyLock<HelpDialog> = LazyLock::new(|| Dialog {
    title: Some(" help "),

    body: INSTRUCTION
        .clone()
        .into_iter()
        .chain([
            DialogLine::new(""),
            DialogLine::new(
                "as a reminder, given `let scan = radar_scan_3x3();`, you get:",
            ),
            DialogLine::new(""),
            DialogLine::new("\t- `scan[0][1]` = tile in front of the bot"),
            DialogLine::new("\t- `scan[1][0]` = tile on bot's left side"),
            DialogLine::new("\t- `scan[1][2]` = tile on bot's right side"),
        ])
        .collect(),

    buttons: vec![HelpDialogResponse::close()],
});

static INSTRUCTION: LazyLock<Vec<DialogLine>> = LazyLock::new(|| {
    vec![
        DialogLine::new(
            "a line following robot does what its name says - it uses radar to \
             check where to go next and then goes there, like:",
        ),
        DialogLine::new(""),
        DialogLine::new("\t1. scan the area"),
        DialogLine::new("\t2a. if there's `'.'` in front you, move there"),
        DialogLine::new("\t2b. or, if there's `'.'` to your left, turn left"),
        DialogLine::new("\t2c. or, if there's `'.'` to your right, turn right"),
        DialogLine::new("\t2d. otherwise stop"),
        DialogLine::new("\t3. go to 1"),
        DialogLine::new(""),
        DialogLine::new("overall, all of those functions should be used:"),
        DialogLine::new(""),
        DialogLine::new("\t- `motor_wait()`"),
        DialogLine::new("\t- `motor_step()`"),
        DialogLine::new("\t- `motor_turn_left()`"),
        DialogLine::new("\t- `motor_turn_right()`"),
        DialogLine::new("\t- `radar_wait()`"),
        DialogLine::new("\t- `radar_scan_3x3()`"),
        DialogLine::new(""),
        DialogLine::new(
            "... and `serial_send_str()` might come handy for debugging!",
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

    ctxt.game
        .update_perms(|perms| {
            perms.user_can_manage_bots = true;
        })
        .await?;

    setup_map(ctxt).await?;

    loop {
        ctxt.wait_until_bot_is_spawned().await?;
        ctxt.game.set_status(Some("WATCHING".into())).await?;

        let outcome = wait(ctxt).await?;

        ctxt.game.set_status(None).await?;

        match outcome {
            Ok(()) => {
                break;
            }

            Err(()) => {
                ctxt.game.run_dialog(&DIALOG_RETRY).await?;
            }
        }
    }

    ctxt.game.set_help(None).await?;

    Ok(())
}

async fn setup_map(ctxt: &mut StepCtxt) -> Result<()> {
    ctxt.world.set_spawn(ivec2(10, 10), Dir::E).await?;

    ctxt.world
        .set_map({
            let mut map = Map::new(uvec2(32, 32));

            map.poly(
                [
                    ivec2(10, 10),
                    ivec2(18, 10),
                    ivec2(18, 9),
                    ivec2(20, 9),
                    ivec2(20, 10),
                    ivec2(28, 10),
                    ivec2(28, 13),
                    ivec2(20, 13),
                    ivec2(20, 14),
                    ivec2(18, 14),
                    ivec2(18, 13),
                    ivec2(10, 13),
                    ivec2(10, 12),
                ],
                TileBase::FLOOR,
            );

            map
        })
        .await?;

    Ok(())
}

async fn wait(ctxt: &mut StepCtxt) -> Result<Result<(), ()>> {
    ctxt.game
        .poll(|ctxt| {
            let Some(bot) = ctxt.world.bots().alive().iter().next() else {
                return Poll::Ready(Err(()));
            };

            if bot.pos == ivec2(10, 12) {
                return Poll::Ready(Ok(()));
            }

            Poll::Pending
        })
        .await
}
