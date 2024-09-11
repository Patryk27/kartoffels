#![feature(let_chains)]

mod bots;
mod driver;
mod drivers;
mod utils;
mod views;

use self::driver::*;
use self::utils::*;
use anyhow::Result;
use glam::uvec2;
use kartoffels_store::Store;
use kartoffels_ui::{Abort, Term};
use std::future::Future;
use std::pin::Pin;
use tokio::select;

pub async fn main(term: &mut Term, store: &Store) -> Result<()> {
    // Wait for terminal size to settle.
    //
    // This matters mostly (only?) for the web, where we initially report (0,0)
    // and then wait for xterm's FitAddon() to kick in and report the actual
    // size.
    while term.size() == uvec2(0, 0) {
        term.poll().await?;
    }

    loop {
        match main_ex(term, store).await {
            Ok(_) => {
                return Ok(());
            }

            Err(err) => {
                if let Some(abort) = err.downcast_ref::<Abort>() {
                    if abort.soft {
                        continue;
                    }
                }

                return Err(err);
            }
        }
    }
}

async fn main_ex(term: &mut Term, store: &Store) -> Result<()> {
    use self::views::*;

    loop {
        let game;
        let driver: Pin<Box<dyn Future<Output = _> + Send>>;
        let mut bg = Background::new(term);

        match home::run(term, store, &mut bg).await? {
            home::Response::Online(world) => {
                let (tx, rx) = DrivenGame::new();

                game = play::run(term, rx);
                driver = Box::pin(drivers::online::run(world, tx));
            }

            home::Response::Sandbox => {
                let (tx, rx) = DrivenGame::new();

                game = play::run(term, rx);
                driver = Box::pin(drivers::sandbox::run(store, tx));
            }

            home::Response::Tutorial => {
                let (tx, rx) = DrivenGame::new();

                game = play::run(term, rx);
                driver = Box::pin(drivers::tutorial::run(store, tx));
            }

            home::Response::Challenges => {
                challenges::run(term, &mut bg).await?;
                continue;
            }

            home::Response::Quit => {
                return Ok(());
            }
        };

        select! {
            result = game => result?,
            result = driver => result?,
        }
    }
}
