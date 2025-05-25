use super::prelude::*;

static MSG: LazyLock<Msg> = LazyLock::new(|| {
    Msg::new("tutorial (12/16)")
        .line("radar provides a scan of the tiles around the bot:")
        .line("")
        .line("    radar_wait(); // wait until radar is ready")
        .line("    radar_scan(3); // do a 3x3 scan")
        .line("")
        .line("    let front = radar_read(0, -1);")
        .line("    let back = radar_read(0, 1);")
        .line("    let left = radar_read(-1, 0);")
        .line("    let right = radar_read(1, 0);")
        .line("")
        .line("    if front == '.' {")
        .line("        // do something")
        .line("    }")
        .line("")
        .line("    if left == '@' || right == '@' {")
        .line("        // do something else")
        .line("    }")
        .line("")
        .line(
            "it's quite configurable - it can scan the directions other bots \
             are facing, their ids etc., look around the api functions if you \
             want to know more",
        )
        .btn(MsgBtn::enter("next", ()))
        .build()
});

pub async fn run(ctxt: &mut TutorialCtxt) -> Result<()> {
    info!("run()");

    ctxt.game.msg(&MSG).await
}
