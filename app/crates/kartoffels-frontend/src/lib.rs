#![feature(let_chains)]

mod utils;
mod views;

use self::utils::*;
use anyhow::Result;
use kartoffels_store::{Session, Store};
use kartoffels_ui::{Abort, Frame};

pub async fn main(
    store: &Store,
    sess: &Session,
    frame: &mut Frame,
) -> Result<()> {
    let mut bg = Background::new(store, frame);

    loop {
        match views::index::run(store, sess, frame, &bg).await {
            Ok(_) => {
                return Ok(());
            }

            Err(err) => {
                match err.downcast::<Abort>() {
                    Ok(abort) => {
                        if abort.soft {
                            // Let soft-aborts generate a new background, just
                            // for fun
                            bg = Background::new(store, frame);
                            continue;
                        } else {
                            return Err(abort.into());
                        }
                    }

                    Err(err) => {
                        views::error::run(frame, &bg, err).await?;
                    }
                }
            }
        }
    }
}

pub fn init() {
    Background::init();
}
