use crate::TestContext;
use kartoffels_bots::CHL_ACYCLIC_MAZE;
use kartoffels_world::prelude::ClockSpeed;
use termwiz::input::KeyCode;

#[tokio::test]
async fn acyclic_maze() {
    let mut ctxt = TestContext::new([]).await;

    ctxt.wait_for(TestContext::HOME).await;
    ctxt.see("[c] challenges");
    ctxt.press(KeyCode::Char('c')).await;

    ctxt.wait_for("[1] acyclic-maze").await;
    ctxt.press(KeyCode::Char('1')).await;

    ctxt.wait_for("[enter] start").await;
    ctxt.see_frame("challenges/acyclic-maze/1.txt").await;

    ctxt.press(KeyCode::Enter).await;
    ctxt.wait_for("upload-bot").await;
    ctxt.wait_while("building world").await;
    ctxt.see_frame("challenges/acyclic-maze/2.txt").await;

    ctxt.store()
        .first_private_world()
        .overclock(ClockSpeed::Unlimited)
        .await
        .unwrap();

    ctxt.upload_bot(CHL_ACYCLIC_MAZE).await;

    ctxt.wait_for("congrats").await;
    ctxt.press(KeyCode::Enter).await;
    ctxt.wait_for("[1] acyclic-maze").await;
}
