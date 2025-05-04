#![feature(associated_type_defaults)]
#![feature(let_chains)]
#![feature(str_as_str)]
#![allow(clippy::needless_pub_self)]

mod frame;
mod theme;
mod ui;
mod utils;
mod views;
mod widgets;

pub use self::frame::*;
pub(self) use self::ui::*;
pub(self) use self::utils::*;
pub(self) use self::widgets::*;
use anyhow::Result;
use kartoffels_store::{Session, Store};

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
