mod intro;
mod world_selection;

use anyhow::Result;
use kartoffels_store::Store;
use kartoffels_ui::Term;
use kartoffels_world::prelude::Handle as WorldHandle;

pub async fn run(term: &mut Term, store: &Store) -> Result<Response> {
    loop {
        match intro::run(term).await? {
            intro::Response::Play => {
                match world_selection::run(term, store).await? {
                    world_selection::Response::Play(world) => {
                        return Ok(Response::Play(world));
                    }

                    world_selection::Response::OpenSandbox => {
                        return Ok(Response::OpenSandbox);
                    }

                    world_selection::Response::OpenTutorial => {
                        return Ok(Response::OpenTutorial);
                    }

                    world_selection::Response::GoBack => {
                        continue;
                    }
                }
            }

            intro::Response::OpenSandbox => {
                return Ok(Response::OpenSandbox);
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
    OpenSandbox,
    OpenTutorial,
    OpenChallenges,
    Quit,
}
