use crate::TestContext;

#[tokio::test]
async fn smoke() {
    TestContext::new([])
        .await
        .see_frame("index/smoke/1.txt")
        .await;
}

#[tokio::test]
async fn too_small_screen() {
    let mut ctxt = TestContext::new_ex(70, 20, []).await;

    ctxt.wait_for("ouch").await;
    ctxt.see_frame("index/too-small-screen/1.txt").await;
}
