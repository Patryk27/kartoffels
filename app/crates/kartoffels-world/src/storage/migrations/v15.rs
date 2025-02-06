use ciborium::Value;

pub fn run(_: &mut Value) {
    panic!(
        "kartoffels v0.7 doesn't support worlds created in v0.6 - please \
         delete all your worlds and create them again",
    );
}
