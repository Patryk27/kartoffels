mod driver;
mod drivers;
mod utils;
mod views;

use self::driver::*;
use self::utils::*;
use self::views::*;
use anyhow::Result;
use kartoffels_store::Store;
use kartoffels_ui::{Abort, Term};
use std::future::Future;
use std::pin::Pin;
use tokio::select;

pub async fn main(term: &mut Term, store: &Store) -> Result<()> {
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
    loop {
        let driver_rx;
        let driver_fut: Pin<Box<dyn Future<Output = _> + Send + Sync>>;

        match home::run(term, store).await? {
            home::Response::Play(world) => {
                let (tx, rx) = DrivenGame::new();

                driver_rx = rx;
                driver_fut = Box::pin(drivers::online::run(world, tx));
            }

            home::Response::Tutorial => {
                let (tx, rx) = DrivenGame::new();

                driver_rx = rx;
                driver_fut = Box::pin(drivers::tutorial::run(store, tx));
            }

            home::Response::Sandbox => {
                let (tx, rx) = DrivenGame::new();

                driver_rx = rx;
                driver_fut = Box::pin(drivers::sandbox::run(tx));
            }

            home::Response::Challenges => {
                todo!();
            }

            home::Response::Quit => {
                return Ok(());
            }
        };

        let game_fut = play::run(term, driver_rx);

        select! {
            result = game_fut => result?,
            result = driver_fut => result?,
        }
    }
}
