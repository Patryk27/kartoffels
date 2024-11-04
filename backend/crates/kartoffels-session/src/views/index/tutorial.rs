mod ctrl;

use crate::views::game;
use anyhow::Result;
use kartoffels_store::{SessionId, Store};
use kartoffels_ui::Term;

pub async fn run(
    store: &Store,
    sess: SessionId,
    term: &mut Term,
) -> Result<()> {
    game::run(store, sess, term, |ctrl| ctrl::run(store, ctrl)).await
}
