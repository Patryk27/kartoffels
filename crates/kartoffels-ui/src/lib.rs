#![feature(impl_trait_in_assoc_type)]

mod components;
mod term;
mod theme;
mod utils;
mod views;

use self::components::*;
pub use self::term::*;
use self::utils::*;
use self::views::*;
use anyhow::Result;
use kartoffels_store::Store;

pub async fn main(term: &mut Term, store: &Store) -> Result<()> {
    loop {
        match home(term, store).await? {
            HomeOutcome::OpenTutorial => {
                todo!();
            }

            HomeOutcome::OpenChallenges => {
                todo!();
            }

            HomeOutcome::Play(world) => {
                play(term, world).await?;
            }

            HomeOutcome::Quit => {
                return Ok(());
            }
        }
    }
}
