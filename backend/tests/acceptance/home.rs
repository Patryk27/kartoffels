use crate::TestContext;

#[tokio::test]
async fn smoke() {
    let mut ctxt = TestContext::new([]).await;

    ctxt.wait_for(TestContext::INDEX).await;
    ctxt.see_frame("home/smoke/1.txt").await;
}

#[tokio::test]
async fn too_small_screen() {
    let mut ctxt = TestContext::new_ex(70, 20, []).await;

    ctxt.wait_for("ouch").await;
    ctxt.see_frame("home/too-small-screen/1.txt").await;
}
