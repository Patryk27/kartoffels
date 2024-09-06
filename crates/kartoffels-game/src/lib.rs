mod driver;
mod drivers;
mod utils;
mod views;

use self::driver::*;
use self::utils::*;
use self::views::*;
use anyhow::Result;
use kartoffels_store::Store;
use kartoffels_ui::Term;
use std::future::Future;
use std::pin::Pin;
use tokio::select;

pub async fn start(term: &mut Term, store: &Store) -> Result<()> {
    loop {
        let driver_rx;
        let driver_fut: Pin<Box<dyn Future<Output = _> + Send + Sync>>;

        match home::run(term, store).await? {
            home::Response::Play(world) => {
                let (tx, rx) = DrivenGame::new();

                driver_rx = rx;
                driver_fut = Box::pin(drivers::online::run(world, tx));
            }

            home::Response::OpenSandbox => {
                let (tx, rx) = DrivenGame::new();

                driver_rx = rx;
                driver_fut = Box::pin(drivers::sandbox::run(tx));
            }

            home::Response::OpenTutorial => {
                let (tx, rx) = DrivenGame::new();

                driver_rx = rx;
                driver_fut = Box::pin(drivers::tutorial::run(store, tx));
            }

            home::Response::OpenChallenges => {
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
