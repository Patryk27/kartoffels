#![feature(impl_trait_in_assoc_type)]
#![feature(let_chains)]

mod term;
mod theme;
mod ui;
mod utils;
mod views;

pub use self::term::*;
use self::ui::*;
use self::utils::*;
use anyhow::Result;
use kartoffels_store::Store;

pub async fn main(term: &mut Term, store: &Store) -> Result<()> {
    use self::views::*;

    loop {
        match home::run(term, store).await? {
            home::Outcome::Play(world) => match play::run(term, world).await? {
                play::Outcome::OpenTutorial => {
                    todo!();
                }

                play::Outcome::Quit => {
                    continue;
                }
            },

            home::Outcome::OpenTutorial => {
                todo!();
            }

            home::Outcome::OpenChallenges => {
                todo!();
            }

            home::Outcome::Quit => {
                return Ok(());
            }
        }
    }
}
