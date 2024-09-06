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
                        return Ok(Response::Sandbox);
                    }

                    world_selection::Response::OpenTutorial => {
                        return Ok(Response::Tutorial);
                    }

                    world_selection::Response::GoBack => {
                        continue;
                    }
                }
            }

            intro::Response::Tutorial => {
                return Ok(Response::Tutorial);
            }

            intro::Response::Sandbox => {
                return Ok(Response::Sandbox);
            }

            intro::Response::Challenges => {
                return Ok(Response::Challenges);
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
    Tutorial,
    Sandbox,
    Challenges,
    Quit,
}
