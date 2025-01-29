mod completed;
mod ctrl;

use crate::views::game;
use anyhow::Result;
use kartoffels_store::{Session, Store};
use kartoffels_ui::Term;

pub async fn run(store: &Store, sess: &Session, term: &mut Term) -> Result<()> {
    let mut completed = false;

    game::run(store, sess, term, |ctrl| {
        ctrl::run(store, ctrl, &mut completed)
    })
    .await?;

    if completed {
        completed::run(term).await?;
    }

    Ok(())
}
