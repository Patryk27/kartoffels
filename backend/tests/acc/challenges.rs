use crate::TestContext;
use termwiz::input::KeyCode;

#[tokio::test]
async fn acyclic_maze() {
    let mut ctxt = TestContext::new().await;

    ctxt.wait_for(TestContext::HOME).await;
    ctxt.see("[c] challenges");
    ctxt.press(KeyCode::Char('c')).await;
    ctxt.wait_for("[1] acyclic-maze").await;
    ctxt.press(KeyCode::Char('1')).await;
    ctxt.wait_for("[u] upload bot").await;
    ctxt.press(KeyCode::Char('u')).await;
    ctxt.upload_bot("bot-chl-acyclic-maze").await;
}
