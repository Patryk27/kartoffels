use crate::TestContext;

#[tokio::test]
async fn smoke() {
    TestContext::new([]).await.see_frame("index/smoke/1.txt");
}

#[tokio::test]
async fn too_small_screen() {
    let mut ctxt = TestContext::new_ex(70, 20, []).await;

    ctxt.wait_for("ouch").await;
    ctxt.see_frame("index/too-small-screen/1.txt");
}

#[tokio::test]
async fn various_screen_sizes() {
    for (w, h) in [(80, 30), (80, 40), (80, 50)] {
        let mut ctxt = TestContext::new_ex(w, h, []).await;

        ctxt.wait_for(TestContext::INDEX).await;
        ctxt.see_frame(format!("index/various-screen-sizes/{w}x{h}.txt"));
    }
}
