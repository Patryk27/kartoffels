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

    let mut bg = Background::new(term);

    loop {
        match main_ex(term, store, &mut bg).await {
            Ok(_) => {
                return Ok(());
            }

            Err(err) => {
                match err.downcast::<Abort>() {
                    Ok(abort) => {
                        if abort.soft {
                            // Let soft-aborts generate a new background, just
                            // for fun
                            bg = Background::new(term);
                            continue;
                        } else {
                            return Err(abort.into());
                        }
                    }

                    Err(err) => {
                        error::run(term, &bg, err).await?;
                    }
                }
            }
        }
    }
}

async fn main_ex(
    term: &mut Term,
    store: &Store,
    bg: &mut Background,
) -> Result<()> {
    loop {
        match home::run(term, store, bg).await? {
            #[allow(clippy::while_let_loop)]
            home::Response::Play => loop {
                match play::run(term, store, bg).await? {
                    play::Response::Play(world) => {
                        drive(term, |game| drivers::online::run(world, game))
                            .await?;
                    }

                    play::Response::GoBack => {
                        break;
                    }
                }
            },

            home::Response::Sandbox => {
                drive(term, |game| drivers::sandbox::run(store, game)).await?;
            }

            home::Response::Tutorial => {
                drive(term, |game| drivers::tutorial::run(store, game)).await?;
            }

            #[allow(clippy::while_let_loop)]
            home::Response::Challenges => loop {
                match challenges::run(term, bg).await? {
                    challenges::Response::Play(challenge) => {
                        drive(term, |game| (challenge.run)(store, game))
                            .await?;
                    }

                    challenges::Response::GoBack => {
                        break;
                    }
                }
            },

            home::Response::Quit => {
                return Ok(());
            }
        };
    }
}

async fn drive<F, Fut>(term: &mut Term, f: F) -> Result<()>
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
