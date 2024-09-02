mod term;
mod theme;
mod ui;
mod utils;
mod views;

pub use self::term::*;
use self::ui::*;
use self::utils::*;
use anyhow::Result;
use kartoffels_store::Store;

pub async fn start(term: &mut Term, store: &Store) -> Result<()> {
    use self::views::*;

    loop {
        let (world, ctrl) = match home::run(term, store).await? {
            home::Response::Play(world) => (world, play::Controller::Normal),

            home::Response::OpenSandbox => {
                (store.sandbox(), play::Controller::Sandbox)
            }

            home::Response::OpenTutorial => {
                todo!();
            }

            home::Response::OpenChallenges => {
                todo!();
            }

            home::Response::Quit => {
                return Ok(());
            }
        };

        match play::run(term, world, ctrl).await? {
            play::Response::OpenTutorial => {
                todo!();
            }

            play::Response::GoBack => {
                continue;
            }
        }
    }
}
