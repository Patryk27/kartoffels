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
    let mut completed = false;

    game::run(store, sess, frame, |ctrl| {
        ctrl::run(store, ctrl, &mut completed)
    })
    .await?;

    if completed {
        completed::run(frame).await?;
    }

    Ok(())
}
