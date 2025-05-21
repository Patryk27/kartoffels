use crate::TestContext;
use kartoffels_prefabs::{ACC_BREAKPOINT, DUMMY};
use kartoffels_world::prelude::{ArenaTheme, Clock, Config, Policy, Theme};
use std::time::Duration;
use termwiz::input::KeyCode;
use tokio::time;

#[tokio::test]
async fn smoke() {
    let mut ctxt = {
        let first = kartoffels_world::create(Config {
            name: "first-world".into(),
            theme: Some(Theme::Arena(ArenaTheme::new(4))),
            ..Default::default()
        });

        let second = kartoffels_world::create(Config {
            name: "second-world".into(),
            policy: Policy {
                max_alive_bots: 1,
                max_queued_bots: 1,
                ..Default::default()
            },
            seed: Some(Default::default()),
            theme: Some(Theme::Arena(ArenaTheme::new(4))),
            ..Default::default()
        });

        let third = kartoffels_world::create(Config {
            name: "third-world".into(),
            theme: Some(Theme::Arena(ArenaTheme::new(4))),
            ..Default::default()
        });

        TestContext::new([first, second, third]).await
    };

    ctxt.see_frame("game/smoke/index-1.txt");

    ctxt.press(KeyCode::Char('p')).await;
    ctxt.wait_for("[1] first-world").await;
    ctxt.see_frame("game/smoke/index-2.txt");

    ctxt.press(KeyCode::Char('2')).await;
    ctxt.wait_for("[j] join-bot").await;
    ctxt.see_frame("game/smoke/game-1.txt");

    ctxt.press(KeyCode::Char('u')).await;
    ctxt.upload_bot(DUMMY).await;
    ctxt.wait_for("> score: 0").await;
    ctxt.see_frame("game/smoke/game-2.txt");

    // ---

    ctxt.press(KeyCode::Char(' ')).await;
    ctxt.wait_for("paused").await;
    ctxt.see_frame("game/smoke/pause-1.txt");

    ctxt.press(KeyCode::Char(' ')).await;
    ctxt.wait_while("paused").await;
    ctxt.see_frame("game/smoke/pause-2.txt");

    // ---

    ctxt.press(KeyCode::Char('h')).await;
    ctxt.wait_for_window("help").await;
    ctxt.see_frame("game/smoke/help-1.txt");

    ctxt.press(KeyCode::Escape).await;
    ctxt.wait_while_modal("help").await;
    ctxt.see_frame("game/smoke/help-2.txt");

    // ---

    ctxt.press(KeyCode::Char('b')).await;
    ctxt.wait_for_window("bots").await;
    ctxt.see_frame("game/smoke/bots-1.txt");

    ctxt.press(KeyCode::Escape).await;
    ctxt.wait_while_modal("bots").await;
    ctxt.see_frame("game/smoke/bots-2.txt");

    // ---

    ctxt.press(KeyCode::Char('f')).await;
    ctxt.wait_for("[f] follow").await;
    ctxt.see_frame("game/smoke/follow-1.txt");

    ctxt.press(KeyCode::Char('f')).await;
    ctxt.wait_for("[f] stop-following").await;
    ctxt.see_frame("game/smoke/follow-2.txt");

    // ---

    ctxt.press(KeyCode::Char('l')).await;
    ctxt.wait_while("[l] leave").await;
    ctxt.see_frame("game/smoke/leave.txt");

    // ---

    ctxt.press(KeyCode::Escape).await;
    ctxt.wait_for_window("exit").await;
    ctxt.see_frame("game/smoke/exit-1.txt");

    ctxt.press(KeyCode::Char('n')).await;
    ctxt.wait_while_modal("exit").await;
    ctxt.see_frame("game/smoke/exit-2.txt");

    ctxt.press(KeyCode::Escape).await;
    ctxt.wait_for_window("exit").await;
    ctxt.press(KeyCode::Char('y')).await;
    ctxt.wait_while_modal("exit").await;
    ctxt.see_frame("game/smoke/exit-3.txt");
}

#[tokio::test]
async fn breakpoint() {
    let mut ctxt = TestContext::new([]).await;

    ctxt.see("[s] sandbox");
    ctxt.press(KeyCode::Char('s')).await;

    ctxt.wait_for("[t] theme").await;
    ctxt.press(KeyCode::Char('t')).await;

    ctxt.wait_for("[a] arena").await;
    ctxt.press(KeyCode::Char('a')).await;

    ctxt.wait_for("[enter] create").await;
    ctxt.press(KeyCode::Enter).await;

    ctxt.wait_for("[u] upload-bot").await;
    ctxt.upload_bot(ACC_BREAKPOINT).await;

    ctxt.wait_for("[l] leave-bot").await;
    ctxt.see_frame("game/breakpoint/1.txt");

    ctxt.world()
        .await
        .overclock(Clock::Unlimited)
        .await
        .unwrap();

    ctxt.wait_for("one").await;
    ctxt.wait_for("breakpoint").await;
    ctxt.see_frame("game/breakpoint/2.txt");

    ctxt.see("[spc] resume");
    ctxt.press(KeyCode::Char(' ')).await;

    ctxt.wait_for("one two").await;
    ctxt.wait_for("breakpoint").await;
    ctxt.see_frame("game/breakpoint/3.txt");
}

#[tokio::test]
async fn http_upload_ok() {
    let mut ctxt = {
        let world = kartoffels_world::create(Config {
            name: "world".into(),
            policy: Policy {
                max_alive_bots: 1,
                max_queued_bots: 1,
                ..Default::default()
            },
            seed: Some(Default::default()),
            ..Default::default()
        });

        TestContext::new([world]).await
    };

    ctxt.see("[p] play");
    ctxt.press(KeyCode::Char('p')).await;

    ctxt.wait_for("[1] world").await;
    ctxt.press(KeyCode::Char('1')).await;

    ctxt.wait_for("[j] join-bot").await;
    ctxt.press(KeyCode::Char('u')).await;

    // Wait until session gets created
    time::sleep(Duration::from_millis(250)).await;

    let sess = ctxt.session().await;

    ctxt.upload_bot_http(sess.id(), DUMMY).await;
    ctxt.wait_for("queued").await;
    ctxt.see_frame("game/http-upload-ok/1.txt");
}

#[tokio::test]
async fn http_upload_err() {
    let mut ctxt = {
        let world = kartoffels_world::create(Config {
            name: "world".into(),
            ..Default::default()
        });

        TestContext::new([world]).await
    };

    ctxt.see("[p] play");
    ctxt.press(KeyCode::Char('p')).await;

    ctxt.wait_for("[1] world").await;
    ctxt.press(KeyCode::Char('1')).await;

    ctxt.wait_for("[j] join-bot").await;
    ctxt.press(KeyCode::Char('u')).await;

    // Wait until session gets created
    time::sleep(Duration::from_millis(250)).await;

    let sess = ctxt.session().await;

    ctxt.upload_bot_http(sess.id(), DUMMY).await;
    ctxt.wait_for_window("ouch").await;
    ctxt.see("couldn't upload bot");
    ctxt.see_frame("game/http-upload-err/1.txt");

    ctxt.press(KeyCode::Enter).await;
    ctxt.wait_while_modal("ouch").await;
    ctxt.see_frame("game/http-upload-err/2.txt");
}

#[tokio::test]
async fn cli_upload_err() {
    let mut ctxt = {
        let world = kartoffels_world::create(Config {
            name: "world".into(),
            ..Default::default()
        });

        TestContext::new([world]).await
    };

    ctxt.see("[p] play");
    ctxt.press(KeyCode::Char('p')).await;

    ctxt.wait_for("[1] world").await;
    ctxt.press(KeyCode::Char('1')).await;

    ctxt.wait_for("[j] join-bot").await;
    ctxt.upload_bot(&[0x00]).await;

    ctxt.wait_for_window("ouch").await;
    ctxt.see("couldn't upload bot");
    ctxt.see_frame("game/cli-upload-err/1.txt");
}
