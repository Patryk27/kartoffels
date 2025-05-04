mod step;
mod steps;

use self::step::*;
use self::steps::*;
use crate::views::game::GameCtrl;
use anyhow::Result;
use kartoffels_store::Store;

pub async fn run(
    store: &Store,
    game: GameCtrl,
    completed: &mut bool,
) -> Result<()> {
    let mut ctxt = TutorialCtxt::new(store, game).await?;

    if !step01::run(&mut ctxt).await? {
        return Ok(());
    }

    step02::run(&mut ctxt).await?;
    step03::run(&mut ctxt).await?;
    step04::run(&mut ctxt).await?;
    step05::run(&mut ctxt).await?;
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

    *completed = true;

    Ok(())
}
