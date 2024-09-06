mod step;
mod steps;

use self::step::*;
use self::steps::*;
use crate::DrivenGame;
use anyhow::Result;
use kartoffels_store::Store;
use std::future;

pub async fn run(store: &Store, game: DrivenGame) -> Result<()> {
    let mut ctxt = StepCtxt {
        store,
        game,
        world: None,
    };

    match step01::run(&mut ctxt).await? {
        step01::Response::Abort => {
            return Ok(());
        }

        step01::Response::Confirm => {
            //
        }
    }

    step02::run(&mut ctxt).await?;
    step03::run(&mut ctxt).await?;
    step04::run(&mut ctxt).await?;
    step05::run(&mut ctxt).await?;
    step06::run(&mut ctxt).await?;
    step07::run(&mut ctxt).await?;
    step08::run(&mut ctxt).await?;

    future::pending().await
}
