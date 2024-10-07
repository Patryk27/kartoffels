use crate::TestContext;

#[tokio::test]
async fn smoke() {
    let mut ctxt = TestContext::new().await;

    ctxt.wait_for(TestContext::HOME).await;
    ctxt.see("[p] play");
    ctxt.see("[s] sandbox");
    ctxt.see("[t] tutorial");
    ctxt.see("[c] challenges");
}
