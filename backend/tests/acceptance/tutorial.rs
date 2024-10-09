use crate::TestContext;
use termwiz::input::{KeyCode, Modifiers};

async fn ctxt() -> TestContext {
    let mut ctxt = TestContext::new().await;

    ctxt.wait_for(TestContext::HOME).await;
    ctxt.see("[t] tutorial");
    ctxt.press(KeyCode::Char('t')).await;

    ctxt.wait_for("ready?").await;
    ctxt.see("[esc] no, leave tutorial");
    ctxt.see("[enter] yes, start tutorial");
    ctxt
}

#[tokio::test]
async fn smoke() {
    let mut ctxt = ctxt().await;

    ctxt.see_frame("tutorial/smoke/1.txt").await;
}

#[tokio::test]
async fn leave() {
    let mut ctxt = ctxt().await;

    ctxt.dont_see(TestContext::HOME);
    ctxt.press(KeyCode::Escape).await;
    ctxt.wait_for(TestContext::HOME).await;
}

#[tokio::test]
async fn leave_and_start() {
    let mut ctxt = ctxt().await;

    ctxt.dont_see(TestContext::HOME);
    ctxt.press(KeyCode::Escape).await;
    ctxt.wait_for(TestContext::HOME).await;

    ctxt.press(KeyCode::Char('t')).await;
    ctxt.wait_for("ready?").await;
    ctxt.see("[esc] no, leave tutorial");
    ctxt.see("[enter] yes, start tutorial");
}

#[tokio::test]
async fn leave_using_ctrl_c() {
    let mut ctxt = ctxt().await;

    ctxt.dont_see(TestContext::HOME);
    ctxt.press_ex(KeyCode::Char('a'), Modifiers::CTRL).await;
    ctxt.wait_for(TestContext::HOME).await;
}

#[tokio::test]
async fn start() {
    let mut ctxt = ctxt().await;

    ctxt.see_frame("tutorial/start/1.txt").await;

    ctxt.press(KeyCode::Enter).await;
    ctxt.wait_for("lesson #1").await;
    ctxt.see_frame("tutorial/start/2.txt").await;
    ctxt.see("[enter] sure");

    ctxt.press(KeyCode::Enter).await;
    ctxt.wait_for("look at you").await;
    ctxt.see_frame("tutorial/start/3.txt").await;
    ctxt.see("[enter] i'm ready");
}
