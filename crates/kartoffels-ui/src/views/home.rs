mod intro;
mod world_selection;

use crate::Term;
use anyhow::Result;
use kartoffels_store::Store;
use kartoffels_world::prelude::Handle as WorldHandle;

pub async fn run(term: &mut Term, store: &Store) -> Result<Response> {
    loop {
        match intro::run(term).await? {
            intro::Response::Play => {
                match world_selection::run(term, store).await? {
                    world_selection::Response::Play(world) => {
                        return Ok(Response::Play(world));
                    }

                    world_selection::Response::Quit => {
                        continue;
                    }
                }
            }

            intro::Response::OpenTutorial => {
                return Ok(Response::OpenTutorial);
            }

            intro::Response::OpenChallenges => {
                return Ok(Response::OpenChallenges);
            }

            intro::Response::Quit => {
                return Ok(Response::Quit);
            }
        }
    }
}

#[derive(Debug)]
pub enum Response {
    Play(WorldHandle),
    OpenTutorial,
    OpenChallenges,
    Quit,
}
