use crate::TestContext;
use kartoffels_prefabs::{TUT_01, TUT_02, TUT_03, TUT_04};
use kartoffels_world::prelude::Clock;
use std::time::Duration;
use termwiz::input::{KeyCode, Modifiers};
use tokio::time;

async fn ctxt() -> TestContext {
    let mut ctxt = TestContext::new([]).await;

    ctxt.see("[t] tutorial");
    ctxt.press(KeyCode::Char('t')).await;

    ctxt.wait_for("hey there").await;
    ctxt.see("[esc] go-back");
    ctxt.see("[enter] start");
    ctxt
}

#[tokio::test]
async fn smoke() {
    let mut ctxt = ctxt().await;

    ctxt.see_frame("tutorial/smoke/1.txt");
}

#[tokio::test]
async fn leave() {
    let mut ctxt = ctxt().await;

    ctxt.dont_see(TestContext::INDEX);
    ctxt.press(KeyCode::Escape).await;
    ctxt.wait_for(TestContext::INDEX).await;
}

#[tokio::test]
async fn leave_and_start() {
    let mut ctxt = ctxt().await;

    ctxt.dont_see(TestContext::INDEX);
    ctxt.press(KeyCode::Escape).await;
    ctxt.wait_for(TestContext::INDEX).await;

    ctxt.press(KeyCode::Char('t')).await;
    ctxt.wait_for("hey there").await;
    ctxt.see("[esc] go-back");
    ctxt.see("[enter] start");
}

#[tokio::test]
async fn leave_using_ctrl_c() {
    let mut ctxt = ctxt().await;

    ctxt.dont_see(TestContext::INDEX);
    ctxt.press_ex(KeyCode::Char('a'), Modifiers::CTRL).await;
    ctxt.wait_for(TestContext::INDEX).await;
}

#[tokio::test]
async fn flow() {
    let mut ctxt = ctxt().await;

    ctxt.see_frame("tutorial/flow/step-01.txt");

    // ---

    ctxt.press(KeyCode::Enter).await;
    ctxt.wait_for_window("tutorial (2/16)").await;
    ctxt.see_frame("tutorial/flow/step-02.txt");

    // ---

    ctxt.press(KeyCode::Enter).await;
    ctxt.wait_for_window("tutorial (3/16)").await;
    ctxt.see_frame("tutorial/flow/step-03.txt");

    // ---

    ctxt.press(KeyCode::Enter).await;
    ctxt.wait_for_window("tutorial (4/16)").await;
    ctxt.see_frame("tutorial/flow/step-04.txt");

    // ---

    ctxt.press(KeyCode::Enter).await;
    ctxt.wait_for_window("tutorial (5/16)").await;
    ctxt.see_frame("tutorial/flow/step-05.txt");

    // ---

    ctxt.press(KeyCode::Enter).await;
    ctxt.wait_while_modal("tutorial (5/16)").await;
    ctxt.see_frame("tutorial/flow/step-05-a.txt");

    ctxt.press(KeyCode::Char('h')).await;
    ctxt.wait_for_window("help").await;
    ctxt.see_frame("tutorial/flow/step-05-b.txt");

    ctxt.press(KeyCode::Escape).await;
    ctxt.wait_while_modal("help").await;
    ctxt.see_frame("tutorial/flow/step-05-c.txt");

    // ---

    ctxt.upload_bot(TUT_01).await;
    ctxt.wait_for_window("tutorial (6/16)").await;
    ctxt.see_frame("tutorial/flow/step-06.txt");

    // ---

    ctxt.press(KeyCode::Enter).await;
    ctxt.wait_for_window("tutorial (7/16)").await;
    ctxt.see_frame("tutorial/flow/step-07-a.txt");

    ctxt.press(KeyCode::Enter).await;
    ctxt.wait_while_modal("tutorial (7/16)").await;
    ctxt.see_frame("tutorial/flow/step-07-b.txt");

    time::pause();
    time::advance(Duration::from_secs(10)).await;
    time::resume();

    // ---

    ctxt.wait_for_window("tutorial (8/16)").await;
    ctxt.see_frame("tutorial/flow/step-08.txt");

    // ---

    ctxt.press(KeyCode::Enter).await;
    ctxt.wait_for_window("tutorial (9/16)").await;
    ctxt.see_frame("tutorial/flow/step-09-a.txt");

    ctxt.press(KeyCode::Enter).await;
    ctxt.wait_while_modal("tutorial (9/16)").await;
    ctxt.see_frame("tutorial/flow/step-09-b.txt");

    ctxt.press(KeyCode::Char('h')).await;
    ctxt.wait_for_window("help").await;
    ctxt.see_frame("tutorial/flow/step-09-c.txt");

    ctxt.press(KeyCode::Escape).await;
    ctxt.wait_while_modal("help").await;
    ctxt.see_frame("tutorial/flow/step-09-d.txt");

    // ---

    ctxt.upload_bot(TUT_01).await;
    ctxt.wait_for_window("tutorial (10/16)").await;
    ctxt.see_frame("tutorial/flow/step-10-a.txt");

    ctxt.press(KeyCode::Enter).await;
    ctxt.wait_while_modal("tutorial (10/16)").await;

    time::pause();
    time::advance(Duration::from_secs(15)).await;
    time::resume();

    ctxt.wait_for_window("tutorial (10/16)").await;
    ctxt.see_frame("tutorial/flow/step-10-b.txt");

    ctxt.press(KeyCode::Enter).await;
    ctxt.wait_while_modal("tutorial (10/16)").await;
    ctxt.see_frame("tutorial/flow/step-10-c.txt");

    ctxt.upload_bot(TUT_02).await;

    ctxt.world()
        .await
        .overclock(Clock::Unlimited)
        .await
        .unwrap();

    // ---

    ctxt.wait_for_window("tutorial (11/16)").await;
    ctxt.see_frame("tutorial/flow/step-11.txt");

    // ---

    ctxt.press(KeyCode::Enter).await;
    ctxt.wait_for_window("tutorial (12/16)").await;
    ctxt.see_frame("tutorial/flow/step-12.txt");

    // ---

    ctxt.press(KeyCode::Enter).await;
    ctxt.wait_for_window("tutorial (13/16)").await;
    ctxt.see_frame("tutorial/flow/step-13-a.txt");

    ctxt.press(KeyCode::Enter).await;
    ctxt.wait_while_modal("tutorial (13/16)").await;
    ctxt.wait_for("......... .........").await;
    ctxt.see_frame("tutorial/flow/step-13-b.txt");

    ctxt.press(KeyCode::Char('h')).await;
    ctxt.wait_for_window("help").await;
    ctxt.see_frame("tutorial/flow/step-13-c.txt");

    ctxt.press(KeyCode::Escape).await;
    ctxt.wait_while_modal("help").await;
    ctxt.see_frame("tutorial/flow/step-13-d.txt");

    ctxt.upload_bot(TUT_02).await;
    ctxt.wait_for_window("tutorial (13/16)").await;
    ctxt.see_frame("tutorial/flow/step-13-e.txt");

    ctxt.press(KeyCode::Enter).await;
    ctxt.wait_while_modal("tutorial (13/16)").await;
    ctxt.see_frame("tutorial/flow/step-13-f.txt");

    ctxt.see("[D] delete-bot");
    ctxt.press(KeyCode::Char('D')).await;
    ctxt.wait_while("[D] delete-bot").await;
    ctxt.see_frame("tutorial/flow/step-13-g.txt");

    ctxt.upload_bot(TUT_03).await;

    ctxt.world()
        .await
        .overclock(Clock::Unlimited)
        .await
        .unwrap();

    // ---

    ctxt.wait_for_window("tutorial (14/16)").await;
    ctxt.see_frame("tutorial/flow/step-14.txt");

    // ---

    ctxt.press(KeyCode::Enter).await;
    ctxt.wait_for_window("tutorial (15/16)").await;
    ctxt.see_frame("tutorial/flow/step-15.txt");

    ctxt.press(KeyCode::Enter).await;

    let mut snapshots = ctxt.world().await.snapshots();

    loop {
        let snapshot = snapshots.next().await.unwrap();

        if snapshot.bots.alive.len() == 10 {
            break;
        }
    }

    time::sleep(Duration::from_millis(250)).await;

    ctxt.upload_bot(TUT_04).await;
    ctxt.wait_for("watching").await;
    ctxt.wait_for("yay, you made it!").await;
    ctxt.see_frame("tutorial/flow/step-16.txt");

    // ---

    ctxt.press(KeyCode::Enter).await;
    ctxt.wait_for(TestContext::INDEX).await;
}
