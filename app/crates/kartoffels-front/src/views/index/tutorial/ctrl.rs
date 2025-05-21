mod step;
mod steps;

use self::step::*;
use self::steps::*;
use crate::views::game::GameCtrl;
use anyhow::Result;
use kartoffels_store::Store;

pub async fn run(store: &Store, game: GameCtrl) -> Result<bool> {
    let mut ctxt = TutorialCtxt::new(store, game).await?;
    let mut step = 1;

    loop {
        let outcome = match step {
            0 => return Ok(false),
            1 => step01::run(&mut ctxt).await?,
            2 => step02::run(&mut ctxt).await?,
            3 => step03::run(&mut ctxt).await?,
            4 => step04::run(&mut ctxt).await?,
            5 => step05::run(&mut ctxt).await?,
            6 => break,
            _ => unreachable!(),
        };

        if outcome {
            step += 1;
        } else {
            step -= 1;
        }
    }

    step06::run(&mut ctxt).await?;
    step07::run(&mut ctxt).await?;
    step08::run(&mut ctxt).await?;
    step09::run(&mut ctxt).await?;
    step10::run(&mut ctxt).await?;
    step11::run(&mut ctxt).await?;
    step12::run(&mut ctxt).await?;
    step13::run(&mut ctxt).await?;
    step14::run(&mut ctxt).await?;
    step15::run(&mut ctxt).await?;

    Ok(true)
}
