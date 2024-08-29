mod intro;
mod world_selection;

use crate::Term;
use anyhow::Result;
use kartoffels_store::Store;
use kartoffels_world::prelude::Handle as WorldHandle;

pub async fn run(term: &mut Term, store: &Store) -> Result<Outcome> {
    loop {
        match intro::run(term).await? {
            intro::Outcome::Play => {
                match world_selection::run(term, store).await? {
                    world_selection::Outcome::Play(world) => {
                        return Ok(Outcome::Play(world));
                    }

                    world_selection::Outcome::Quit => {
                        continue;
                    }
                }
            }

            intro::Outcome::SeeTutorial => {
                return Ok(Outcome::OpenTutorial);
            }

            intro::Outcome::SeeChallenges => {
                return Ok(Outcome::OpenChallenges);
            }

            intro::Outcome::Quit => {
                return Ok(Outcome::Quit);
            }
        }
    }
}

#[derive(Debug)]
pub enum Outcome {
    Play(WorldHandle),
    OpenTutorial,
    OpenChallenges,
    Quit,
}
