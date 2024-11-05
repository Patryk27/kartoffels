use crate::TestContext;
use kartoffels_world::prelude::{Config, Policy};
use termwiz::input::KeyCode;

#[tokio::test]
async fn smoke() {
    let mut ctxt = {
        let first = kartoffels_world::create(Config {
            name: "first world".into(),
            ..Default::default()
        });

        let second = kartoffels_world::create(Config {
            name: "second world".into(),
            policy: Policy {
                max_alive_bots: 1,
                max_queued_bots: 1,
                ..Default::default()
            },
            rng: Some(Default::default()),
            ..Default::default()
        });

        let third = kartoffels_world::create(Config {
            name: "third world".into(),
            ..Default::default()
        });

        TestContext::new([first, second, third]).await
    };

    ctxt.wait_for(TestContext::HOME).await;
    ctxt.see_frame("game/smoke/home-1.txt").await;

    ctxt.press(KeyCode::Char('p')).await;
    ctxt.wait_for("[1] first world").await;
    ctxt.see_frame("game/smoke/home-2.txt").await;

    ctxt.press(KeyCode::Char('2')).await;
    ctxt.wait_for("[j] join bot").await;
    ctxt.see_frame("game/smoke/game-1.txt").await;

    ctxt.press(KeyCode::Char('u')).await;
    ctxt.upload_bot("dummy").await;
    ctxt.wait_for("[l] leave").await;
    ctxt.see_frame("game/smoke/game-2.txt").await;

    // ---

    ctxt.press(KeyCode::Char(' ')).await;
    ctxt.wait_for("PAUSED").await;
    ctxt.see_frame("game/smoke/pause-1.txt").await;

    ctxt.press(KeyCode::Char(' ')).await;
    ctxt.wait_while("PAUSED").await;
    ctxt.see_frame("game/smoke/pause-2.txt").await;

    // ---

    ctxt.press(KeyCode::Char('h')).await;
    ctxt.wait_for_modal("help").await;
    ctxt.see_frame("game/smoke/help-1.txt").await;

    ctxt.press(KeyCode::Escape).await;
    ctxt.wait_while_modal("help").await;
    ctxt.see_frame("game/smoke/help-2.txt").await;

    // ---

    ctxt.press(KeyCode::Char('b')).await;
    ctxt.wait_for_modal("bots").await;
    ctxt.see_frame("game/smoke/bots-1.txt").await;

    ctxt.press(KeyCode::Escape).await;
    ctxt.wait_while_modal("bots").await;
    ctxt.see_frame("game/smoke/bots-2.txt").await;

    // ---

    ctxt.press(KeyCode::Char('f')).await;
    ctxt.wait_for("[f] follow").await;
    ctxt.see_frame("game/smoke/follow-1.txt").await;

    ctxt.press(KeyCode::Char('f')).await;
    ctxt.wait_for("[f] stop following").await;
    ctxt.see_frame("game/smoke/follow-2.txt").await;

    // ---

    ctxt.press(KeyCode::Char('l')).await;
    ctxt.wait_while("[l] leave").await;
    ctxt.see_frame("game/smoke/leave.txt").await;

    // ---

    ctxt.press(KeyCode::Escape).await;
    ctxt.wait_for_modal("go back").await;
    ctxt.see_frame("game/smoke/go-back-1.txt").await;

    ctxt.press(KeyCode::Char('n')).await;
    ctxt.wait_while_modal("go back").await;
    ctxt.see_frame("game/smoke/go-back-2.txt").await;

    ctxt.press(KeyCode::Escape).await;
    ctxt.wait_for_modal("go back").await;
    ctxt.press(KeyCode::Char('y')).await;
    ctxt.wait_while_modal("go back").await;
    ctxt.see_frame("game/smoke/go-back-3.txt").await;
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
            rng: Some(Default::default()),
            ..Default::default()
        });

        TestContext::new([world]).await
    };

    ctxt.wait_for(TestContext::HOME).await;
    ctxt.press(KeyCode::Char('p')).await;

    ctxt.wait_for("[1] world").await;
    ctxt.press(KeyCode::Char('1')).await;

    ctxt.wait_for("[j] join bot").await;
    ctxt.press(KeyCode::Char('u')).await;

    let sess = ctxt.store().first_session_id();

    ctxt.upload_bot_http(sess, "dummy").await;
    ctxt.wait_for("[l] leave").await;
    ctxt.see_frame("game/http-upload-ok/1.txt").await;
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

    ctxt.wait_for(TestContext::HOME).await;
    ctxt.press(KeyCode::Char('p')).await;

    ctxt.wait_for("[1] world").await;
    ctxt.press(KeyCode::Char('1')).await;

    ctxt.wait_for("[j] join bot").await;
    ctxt.press(KeyCode::Char('u')).await;

    let sess = ctxt.store().first_session_id();

    ctxt.upload_bot_http(sess, "dummy").await;
    ctxt.wait_for_modal("ouch").await;
    ctxt.see_frame("game/http-upload-err/1.txt").await;

    ctxt.press(KeyCode::Enter).await;
    ctxt.wait_while_modal("ouch").await;
    ctxt.see_frame("game/http-upload-err/2.txt").await;
}
