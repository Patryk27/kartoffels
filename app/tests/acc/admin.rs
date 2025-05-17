use crate::TestContext;

#[tokio::test]
async fn help() {
    TestContext::new([])
        .await
        .run_and_see("--help", "admin/help/stdout.txt")
        .await;
}

#[tokio::test]
async fn world_mgmt() {
    let ctxt = TestContext::new([]).await;

    for (idx, name) in [
        "tung-tung-tung-sahur",
        "balerrina-capuccina",
        "tralalero-tralala",
    ]
    .into_iter()
    .enumerate()
    {
        let cmd = if idx == 1 {
            format!(
                "world create \
                 {name} \
                 --theme '{{ \
                     \"type\": \"arena\", \
                     \"radius\": 16 \
                 }}'",
            )
        } else {
            format!(
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
            )
        };

        ctxt.run_and_see(
            cmd,
            format!("admin/world-mgmt/1{}.txt", (b'a' + idx as u8) as char),
        )
        .await;
    }

    // ---

    ctxt.run_and_see("world list", "admin/world-mgmt/2.txt")
        .await;

    // ---

    ctxt.run_and_see(
        "world get 0000-0000-0000-0001",
        "admin/world-mgmt/3a.txt",
    )
    .await;

    ctxt.run_and_see(
        "world get 0000-0000-0000-0002",
        "admin/world-mgmt/3b.txt",
    )
    .await;

    ctxt.run_and_see(
        "world get 0000-0000-0000-0003",
        "admin/world-mgmt/3c.txt",
    )
    .await;

    // ---

    ctxt.run_and_see(
        "world rename 0000-0000-0000-0002 something-else",
        "admin/world-mgmt/4a.txt",
    )
    .await;

    ctxt.run_and_see("world list", "admin/world-mgmt/4b.txt")
        .await;

    // ---

    ctxt.run_and_see(
        "world delete 0000-0000-0000-0002",
        "admin/world-mgmt/5a.txt",
    )
    .await;

    ctxt.run_and_see("world list", "admin/world-mgmt/5b.txt")
        .await;
}
