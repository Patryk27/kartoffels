use crate::TestContext;
use kartoffels_bots::{CHL_ACYCLIC_MAZE, CHL_DIAMOND_HEIST};
use kartoffels_world::prelude::ClockSpeed;
use termwiz::input::KeyCode;

#[tokio::test]
async fn acyclic_maze() {
    let mut ctxt = TestContext::new([]).await;

    ctxt.wait_for(TestContext::INDEX).await;
    ctxt.see("[c] challenges");
    ctxt.press(KeyCode::Char('c')).await;

    ctxt.wait_for("[a] acyclic-maze").await;
    ctxt.press(KeyCode::Char('a')).await;

    ctxt.wait_for("[enter] start").await;
    ctxt.see_frame("challenges/acyclic-maze/1.txt").await;

    // ---

    ctxt.press(KeyCode::Enter).await;
    ctxt.wait_for("upload-bot").await;
    ctxt.wait_while("building-world").await;
    ctxt.see_frame("challenges/acyclic-maze/2.txt").await;

    ctxt.upload_bot(CHL_ACYCLIC_MAZE).await;

    ctxt.store()
        .first_private_world()
        .overclock(ClockSpeed::Unlimited)
        .await
        .unwrap();

    ctxt.wait_for("congrats").await;
    ctxt.press(KeyCode::Enter).await;

    // ---

    ctxt.wait_for("[a] acyclic-maze").await;
}

#[tokio::test]
async fn diamond_heist() {
    let mut ctxt = TestContext::new([]).await;

    ctxt.wait_for(TestContext::INDEX).await;
    ctxt.see("[c] challenges");
    ctxt.press(KeyCode::Char('c')).await;

    ctxt.wait_for("[d] diamond-heist").await;
    ctxt.press(KeyCode::Char('d')).await;

    ctxt.wait_for("[enter] start").await;
    ctxt.see_frame("challenges/diamond-heist/1.txt").await;

    // ---

    ctxt.press(KeyCode::Enter).await;
    ctxt.wait_for("upload-bot").await;
    ctxt.wait_while("building-world").await;
    ctxt.see_frame("challenges/diamond-heist/2.txt").await;

    ctxt.upload_bot(CHL_DIAMOND_HEIST).await;

    ctxt.store()
        .first_private_world()
        .overclock(ClockSpeed::Unlimited)
        .await
        .unwrap();

    ctxt.wait_for("congrats").await;
    ctxt.press(KeyCode::Enter).await;

    // ---

    ctxt.wait_for("[d] diamond-heist").await;
}
