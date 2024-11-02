#![feature(let_chains)]

mod driver;
mod drivers;
mod utils;
mod views;

use self::driver::*;
use self::utils::*;
use self::views::*;
use anyhow::Result;
use kartoffels_store::{SessionId, SessionToken, Store};
use kartoffels_ui::{Abort, Term};
use std::future::Future;
use tokio::select;

pub async fn main(
    store: &Store,
    sess: &SessionToken,
    term: &mut Term,
) -> Result<()> {
    let mut bg = Background::new(store, term);

    loop {
        match main_ex(store, sess.id(), term, &mut bg).await {
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
                        error::run(term, &bg, err).await?;
                    }
                }
            }
        }
    }
}

async fn main_ex(
    store: &Store,
    sess: SessionId,
    term: &mut Term,
    bg: &mut Background,
) -> Result<()> {
    let mut fade_in = true;

    loop {
        match home::run(store, term, bg, fade_in).await? {
            #[allow(clippy::while_let_loop)]
            home::Response::Play => {
                fade_in = false;

                loop {
                    match play::run(store, term, bg, fade_in).await? {
                        play::Response::Play(world) => {
                            drive(store, sess, term, |game| {
                                drivers::online::run(world, game)
                            })
                            .await?;

                            fade_in = true;
                            continue;
                        }

                        play::Response::GoBack => {
                            fade_in = false;
                            break;
                        }
                    }
                }
            }

            home::Response::Sandbox => {
                drive(store, sess, term, |game| {
                    drivers::sandbox::run(store, game)
                })
                .await?;

                fade_in = true;
                continue;
            }

            home::Response::Tutorial => {
                drive(store, sess, term, |game| {
                    drivers::tutorial::run(store, game)
                })
                .await?;

                fade_in = true;
                continue;
            }

            #[allow(clippy::while_let_loop)]
            home::Response::Challenges => {
                fade_in = false;

                loop {
                    match challenges::run(store, term, bg, fade_in).await? {
                        challenges::Response::Play(challenge) => {
                            drive(store, sess, term, |game| {
                                (challenge.run)(store, game)
                            })
                            .await?;

                            fade_in = true;
                            continue;
                        }

                        challenges::Response::GoBack => {
                            fade_in = false;
                            break;
                        }
                    }
                }
            }

            home::Response::Quit => {
                return Ok(());
            }
        };
    }
}

async fn drive<F, Fut>(
    store: &Store,
    sess: SessionId,
    term: &mut Term,
    f: F,
) -> Result<()>
where
    F: FnOnce(DrivenGame) -> Fut,
    Fut: Future<Output = Result<()>>,
{
    let (tx, rx) = DrivenGame::new();
    let view = Box::pin(game::run(store, sess, term, rx));
    let driver = Box::pin(f(tx));

    select! {
        result = view => result,
        result = driver => result,
    }
}
