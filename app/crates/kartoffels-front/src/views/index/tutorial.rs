mod ctrl;
mod end;

use crate::views::game;
use crate::Frame;
use anyhow::Result;
use kartoffels_store::{Session, Store};

pub async fn run(
    store: &Store,
    sess: &Session,
    frame: &mut Frame,
) -> Result<()> {
    let finished =
        game::run(store, sess, frame, |ctrl| ctrl::run(store, ctrl)).await?;

    if finished.unwrap_or(false) {
        end::run(store, frame).await?;
    }

    Ok(())
}
