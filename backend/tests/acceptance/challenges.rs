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
    ctxt.wait_for("[enter] let's do it").await;
    ctxt.see_frame("challenges/acyclic-maze/1.txt").await;
    ctxt.press(KeyCode::Enter).await;
    ctxt.wait_for("BUILDING WORLD").await;
    ctxt.wait_while("BUILDING WORLD").await;
    ctxt.see_frame("challenges/acyclic-maze/2.txt").await;
}
