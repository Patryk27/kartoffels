#![feature(let_chains)]

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
use tokio::select;
use tracing::info;

pub async fn main(term: &mut Term, store: &Store) -> Result<()> {
    info!("session started");

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
        let mut bg = Background::new(term);

        match home::run(term, &mut bg).await? {
            home::Response::Play => loop {
                match play::run(term, store, &mut bg).await? {
                    play::Response::Play(world) => {
                        run_game(term, |game| {
                            drivers::online::run(world, game)
                        })
                        .await?;
                    }

                    play::Response::Sandbox => {
                        run_game(term, |game| {
                            drivers::sandbox::run(store, game)
                        })
                        .await?;
                    }

                    play::Response::Challenges => {
                        challenges::run(term, &mut bg).await?;
                    }

                    play::Response::GoBack => {
                        break;
                    }
                }
            },

            home::Response::Tutorial => {
                run_game(term, |game| drivers::tutorial::run(store, game))
                    .await?;
            }

            home::Response::Quit => {
                return Ok(());
            }
        };
    }
}

async fn run_game<F, Fut>(term: &mut Term, f: F) -> Result<()>
where
    F: FnOnce(DrivenGame) -> Fut,
    Fut: Future<Output = Result<()>>,
{
    let (tx, rx) = DrivenGame::new();
    let view = Box::pin(game::run(term, rx));
    let driver = Box::pin(f(tx));

    select! {
        result = view => result,
        result = driver => result,
    }
}
