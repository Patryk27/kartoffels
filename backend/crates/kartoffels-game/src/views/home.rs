mod background;
mod intro;
mod world_selection;

use self::background::*;
use anyhow::Result;
use kartoffels_store::Store;
use kartoffels_ui::Term;
use kartoffels_world::prelude::Handle as WorldHandle;

pub async fn run(term: &mut Term, store: &Store) -> Result<Response> {
    let mut bg = Background::new(term);

    loop {
        match intro::run(term, store, &mut bg).await? {
            intro::Response::OnlinePlay => {
                match world_selection::run(term, store, &mut bg).await? {
                    world_selection::Response::OnlinePlay(world) => {
                        return Ok(Response::Play(world));
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
