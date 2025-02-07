use crate::TestContext;
use termwiz::input::{KeyCode, Modifiers};

async fn logged_ctxt() -> TestContext {
    let mut ctxt = TestContext::new([]).await;

    ctxt.press_ex(KeyCode::Char('x'), Modifiers::ALT).await;
    ctxt.wait_for("enter secret").await;

    ctxt.write("foobar").await;
    ctxt.press(KeyCode::Enter).await;
    ctxt.wait_for_window("console").await;
    ctxt
}

#[tokio::test]
async fn login_ok() {
    let mut ctxt = TestContext::new([]).await;

    ctxt.press_ex(KeyCode::Char('x'), Modifiers::ALT).await;
    ctxt.wait_for("enter secret").await;
    ctxt.see_frame("console/login-ok/login-1.txt").await;

    ctxt.write("foobar").await;
    ctxt.press(KeyCode::Enter).await;
    ctxt.wait_for_window("console").await;
    ctxt.see_frame("console/login-ok/login-2.txt").await;
}

#[tokio::test]
async fn login_err() {
    let mut ctxt = TestContext::new([]).await;

    ctxt.press_ex(KeyCode::Char('x'), Modifiers::ALT).await;
    ctxt.wait_for("enter secret").await;
    ctxt.see_frame("console/login-err/1.txt").await;

    ctxt.write("foobars").await;
    ctxt.press(KeyCode::Enter).await;
    ctxt.wait_for("invalid secret").await;
    ctxt.see_frame("console/login-err/2.txt").await;

    ctxt.press(KeyCode::Enter).await;
    ctxt.wait_while("invalid secret").await;
    ctxt.see_frame("console/login-err/3.txt").await;
}

#[tokio::test]
async fn cmd_exit() {
    let mut ctxt = logged_ctxt().await;

    ctxt.write("exit").await;
    ctxt.press(KeyCode::Enter).await;
    ctxt.wait_for(TestContext::INDEX).await;
}

#[tokio::test]
async fn cmd_help() {
    let mut ctxt = logged_ctxt().await;

    ctxt.write("help").await;
    ctxt.press(KeyCode::Enter).await;
    ctxt.wait_for("; help").await;
    ctxt.see_frame("console/cmd-help/1.txt").await;
}

#[tokio::test]
async fn cmd_invalid() {
    let mut ctxt = logged_ctxt().await;

    ctxt.write("helpo").await;
    ctxt.press(KeyCode::Enter).await;
    ctxt.wait_for("; helpo").await;
    ctxt.see_frame("console/cmd-invalid/1.txt").await;
}

#[tokio::test]
async fn cmd_world_mgmt() {
    let mut ctxt = logged_ctxt().await;

    ctxt.write("create-world foo --theme arena:radius=8").await;
    ctxt.press(KeyCode::Enter).await;
    ctxt.wait_for("id: 0000-0000-0000-0001").await;

    ctxt.write("create-world zar --theme arena:radius=8").await;
    ctxt.press(KeyCode::Enter).await;
    ctxt.wait_for("id: 0000-0000-0000-0002").await;

    ctxt.write("create-world bar --theme arena:radius=8").await;
    ctxt.press(KeyCode::Enter).await;
    ctxt.wait_for("id: 0000-0000-0000-0003").await;

    ctxt.write("list-worlds").await;
    ctxt.press(KeyCode::Enter).await;
    ctxt.wait_for("; list-worlds").await;

    ctxt.see_frame("console/cmd-world-mgmt/1.txt").await;

    // ---

    ctxt.write("exit").await;
    ctxt.press(KeyCode::Enter).await;
    ctxt.wait_for(TestContext::INDEX).await;

    // ---

    ctxt.see("[p] play");
    ctxt.press(KeyCode::Char('p')).await;
    ctxt.wait_for_window("play").await;
    ctxt.see_frame("console/cmd-world-mgmt/2.txt").await;
}
