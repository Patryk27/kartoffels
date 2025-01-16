#![feature(let_chains)]

mod utils;
mod views;

use self::utils::*;
use anyhow::Result;
use kartoffels_store::{SessionToken, Store};
use kartoffels_ui::{Abort, Term};

pub async fn main(
    store: &Store,
    sess: &SessionToken,
    term: &mut Term,
) -> Result<()> {
    let mut bg = Background::new(store, term);

    loop {
        match views::index::run(store, sess.id(), term, &mut bg).await {
            Ok(_) => {
                return Ok(());
            }

            Err(err) => {
                match err.downcast::<Abort>() {
                    Ok(abort) => {
                        if abort.soft {
                            // Let soft-aborts generate a new background, just
                            // for fun
                            bg = Background::new(store, term);
                            continue;
                        } else {
                            return Err(abort.into());
                        }
                    }

                    Err(err) => {
                        views::error::run(term, &bg, err).await?;
                    }
                }
            }
        }
    }
}
