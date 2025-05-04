use crate::TestContext;

#[tokio::test]
async fn help() {
    let ctxt = TestContext::new([]).await;
    let out = ctxt.cmd("--help").await;

    ctxt.asserter("admin/help").assert("stdout.txt", &out);
}

#[tokio::test]
async fn world_mgmt() {
    let ctxt = TestContext::new([]).await;

    // ---

    for (idx, name) in [
        "tung-tung-tung-sahur",
        "balerrina-capuccina",
        "tralalero-tralala",
    ]
    .into_iter()
    .enumerate()
    {
        let out = ctxt
            .cmd(format!(
                "world create \
                 {name} \
                 --theme '{{ \
                     \"type\": \"arena\", \
                     \"radius\": 16 \
                 }}' \
                 --policy '{{ \
                     \"auto_respawn\": false, \
                     \"max_alive_bots\": 32, \
                     \"max_queued_bots\": 64 \
                 }}'",
            ))
            .await;

        ctxt.asserter("admin/world-mgmt")
            .assert(format!("{}.txt", idx + 1), &out);
    }

    // ---

    let out = ctxt.cmd("world list").await;

    ctxt.asserter("admin/world-mgmt").assert("4.txt", &out);

    // ---

    let out = ctxt
        .cmd("world rename 0000-0000-0000-0002 something-else")
        .await;

    ctxt.asserter("admin/world-mgmt").assert("5.txt", &out);

    // ---

    let out = ctxt.cmd("world list").await;

    ctxt.asserter("admin/world-mgmt").assert("6.txt", &out);

    // ---

    let out = ctxt.cmd("world delete 0000-0000-0000-0002").await;

    ctxt.asserter("admin/world-mgmt").assert("7.txt", &out);

    // ---

    let out = ctxt.cmd("world list").await;

    ctxt.asserter("admin/world-mgmt").assert("8.txt", &out);
}
