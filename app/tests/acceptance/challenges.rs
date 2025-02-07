use crate::TestContext;
use kartoffels_prefabs::{
    CHL_ACYCLIC_MAZE, CHL_DIAMOND_HEIST, CHL_PERSONAL_ROOMBA,
};
use kartoffels_world::prelude::Clock;
use termwiz::input::KeyCode;

#[tokio::test]
async fn acyclic_maze() {
    let mut ctxt = TestContext::new([]).await;

    ctxt.see("[c] challenges");
    ctxt.press(KeyCode::Char('c')).await;
    ctxt.wait_for("[a] acyclic-maze").await;

    ctxt.press(KeyCode::Char('a')).await;
    ctxt.wait_for_window("acyclic-maze").await;
    ctxt.see_frame("challenges/acyclic-maze/1.txt").await;

    // ---

    ctxt.press(KeyCode::Enter).await;
    ctxt.wait_while_modal("acyclic-maze").await;
    ctxt.wait_for("building").await;
    ctxt.wait_while("building").await;
    ctxt.see_frame("challenges/acyclic-maze/2.txt").await;

    // ---

    let world = ctxt.store().first_private_world();

    world.overclock(Clock::manual()).await.unwrap();
    ctxt.upload_bot(CHL_ACYCLIC_MAZE).await;
    ctxt.wait_while("[u] upload-bot").await;

    world.tick(10_000_000).await.unwrap();

    ctxt.sync(world.version()).await;
    ctxt.wait_for("congrats").await;
    ctxt.see_frame("challenges/acyclic-maze/3.txt").await;

    // ---

    ctxt.press(KeyCode::Enter).await;
    ctxt.wait_for("[a] acyclic-maze").await;
}

#[tokio::test]
async fn diamond_heist() {
    let mut ctxt = TestContext::new([]).await;

    ctxt.see("[c] challenges");
    ctxt.press(KeyCode::Char('c')).await;
    ctxt.wait_for("[d] diamond-heist").await;

    ctxt.press(KeyCode::Char('d')).await;
    ctxt.wait_for_window("diamond-heist").await;
    ctxt.see_frame("challenges/diamond-heist/1.txt").await;

    // ---

    ctxt.press(KeyCode::Enter).await;
    ctxt.wait_while_modal("diamond-heist").await;
    ctxt.wait_for("building").await;
    ctxt.wait_while("building").await;
    ctxt.see_frame("challenges/diamond-heist/2.txt").await;

    // ---

    let world = ctxt.store().first_private_world();

    ctxt.upload_bot(CHL_DIAMOND_HEIST).await;
    ctxt.wait_for("id").await;

    world.tick(1_500_000).await.unwrap();

    ctxt.sync(world.version()).await;
    ctxt.see_frame("challenges/diamond-heist/3.txt").await;

    // ---

    ctxt.press(KeyCode::Enter).await;
    ctxt.wait_for("[d] diamond-heist").await;
}

#[tokio::test]
async fn personal_roomba() {
    let mut ctxt = TestContext::new([]).await;

    ctxt.see("[c] challenges");
    ctxt.press(KeyCode::Char('c')).await;
    ctxt.wait_for("[p] personal-roomba").await;

    ctxt.press(KeyCode::Char('p')).await;
    ctxt.wait_for_window("personal-roomba").await;
    ctxt.see_frame("challenges/personal-roomba/1.txt").await;

    // ---

    ctxt.press(KeyCode::Enter).await;
    ctxt.wait_while_modal("personal-roomba").await;
    ctxt.wait_for("building").await;
    ctxt.wait_while("building").await;
    ctxt.see_frame("challenges/personal-roomba/2.txt").await;

    let world = ctxt.store().first_private_world();

    world.overclock(Clock::manual()).await.unwrap();
    ctxt.upload_bot(CHL_PERSONAL_ROOMBA).await;
    ctxt.wait_while("[u] upload-bot").await;

    world.tick(10_000_000).await.unwrap();

    ctxt.sync(world.version()).await;
    ctxt.wait_for("congrats").await;
    ctxt.see_frame("challenges/personal-roomba/3.txt").await;

    // ---

    ctxt.press(KeyCode::Enter).await;
    ctxt.wait_for("[p] personal-roomba").await;
}
