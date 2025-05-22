mod completed;
mod ctrl;

use crate::views::game;
use crate::Frame;
use anyhow::Result;
use kartoffels_store::{Session, Store};

pub async fn run(
    store: &Store,
    sess: &Session,
    frame: &mut Frame,
) -> Result<()> {
    let completed =
        game::run(store, sess, frame, |ctrl| ctrl::run(store, ctrl)).await?;

    if completed.unwrap_or(false) {
        completed::run(store, frame).await?;
    }

    Ok(())
}
