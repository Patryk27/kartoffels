use crate::TestContext;

#[tokio::test]
async fn smoke() {
    let mut ctxt = TestContext::new().await;

    ctxt.wait_for(TestContext::HOME).await;
    ctxt.dont_see("[p] play");
    ctxt.see("[s] sandbox");
    ctxt.see("[t] tutorial");
    ctxt.see("[c] challenges");
    ctxt.dont_see("[esc] quit");
}

#[tokio::test]
async fn too_small_screen() {
    let mut ctxt = TestContext::new_ex(70, 20).await;

    ctxt.wait_for("ouch").await;
    ctxt.see_frame("home/too-small-screen/1.txt").await;
}
