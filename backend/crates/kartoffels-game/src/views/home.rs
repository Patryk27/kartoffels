mod intro;
mod world_selection;

use crate::Background;
use anyhow::Result;
use kartoffels_store::Store;
use kartoffels_ui::Term;
use kartoffels_world::prelude::Handle as WorldHandle;

pub async fn run(
    term: &mut Term,
    store: &Store,
    bg: &mut Background,
) -> Result<Response> {
    loop {
        match intro::run(term, store, bg).await? {
            intro::Response::Online => {
                if let world_selection::Response::Some(world) =
                    world_selection::run(term, store, bg).await?
                {
                    return Ok(Response::Online(world));
                } else {
                    continue;
                }
            }

            intro::Response::Sandbox => {
                return Ok(Response::Sandbox);
            }

            intro::Response::Tutorial => {
                return Ok(Response::Tutorial);
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
    Online(WorldHandle),
    Sandbox,
    Tutorial,
    Challenges,
    Quit,
}
