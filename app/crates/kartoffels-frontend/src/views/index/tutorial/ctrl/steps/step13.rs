use super::prelude::*;

static MSG: LazyLock<Msg> = LazyLock::new(|| Msg {
    title: Some(" tutorial (13/16) "),

    body: vec![
        MsgLine::new(
            "so, how about we solidify all this knowledge by implementing a \
             good-old *line follower*, eh?",
        ),
        MsgLine::new(""),
        MsgLine::new(
            "ekhm, i'm sorry for my sudden canadian accent, don't know what \
             happened",
        ),
        MsgLine::new(""),
    ]
    .into_iter()
    .chain(DOCS.clone())
    .collect(),

    buttons: vec![MsgButton::confirm("next", ())],
});

static HELP: LazyLock<HelpMsg> = LazyLock::new(|| Msg {
    title: Some(" help "),
    body: DOCS.clone(),
    buttons: vec![HelpMsgEvent::close()],
});

static DOCS: LazyLock<Vec<MsgLine>> = LazyLock::new(|| {
    vec![
        MsgLine::new(
            "a line follower uses radar to check where to go next and then \
             goes there, like:",
        ),
        MsgLine::new(""),
        MsgLine::new("\t1. scan the area"),
        MsgLine::new("\t2a. if there's `'.'` in front you, move there"),
        MsgLine::new("\t2b. or, if there's `'.'` to your left, turn left"),
        MsgLine::new("\t2c. or, if there's `'.'` to your right, turn right"),
        MsgLine::new("\t2d. otherwise stop"),
        MsgLine::new("\t3. go to 1"),
        MsgLine::new(""),
        MsgLine::new("overall, all of those functions should be used:"),
        MsgLine::new(""),
        MsgLine::new("\t- `motor_wait()`"),
        MsgLine::new("\t- `motor_step()`"),
        MsgLine::new("\t- `motor_turn_left()`"),
        MsgLine::new("\t- `motor_turn_right()`"),
        MsgLine::new("\t- `radar_wait()`"),
        MsgLine::new("\t- `radar_scan_3x3()`"),
        MsgLine::new(""),
        MsgLine::new("... and `println!()` might come handy for debugging!"),
    ]
});

static MSG_RETRY: LazyLock<Msg> = LazyLock::new(|| Msg {
    title: Some(" tutorial (13/16) "),
    body: vec![MsgLine::new(
        "hmm, your robot seems to have died — delete it and upload something \
         better, i know you have it in you",
    )],
    buttons: vec![MsgButton::confirm("try-again", ())],
});

pub async fn run(ctxt: &mut TutorialCtxt) -> Result<()> {
    ctxt.game.msg(&MSG).await?;
    ctxt.game.set_help(Some(&HELP)).await?;

    setup_map(ctxt).await?;

    loop {
        ctxt.events.next_born_bot().await?;
        ctxt.game.set_status(Some("watching".into())).await?;

        let succeeded = wait(ctxt).await?;

        ctxt.game.set_status(None).await?;

        if succeeded {
            break;
        } else {
            ctxt.game.msg(&MSG_RETRY).await?;
        }
    }

    ctxt.game.set_help(None).await?;

    Ok(())
}

async fn setup_map(ctxt: &mut TutorialCtxt) -> Result<()> {
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
                TileKind::FLOOR,
            );

            map
        })
        .await?;

    Ok(())
}

async fn wait(ctxt: &mut TutorialCtxt) -> Result<bool> {
    loop {
        if let Some(bot) = ctxt.snapshots.next().await?.bots.alive.iter().next()
        {
            if bot.pos == ivec2(10, 12) {
                return Ok(true);
            }
        } else {
            return Ok(false);
        }
    }
}
