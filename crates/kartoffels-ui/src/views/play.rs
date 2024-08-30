mod bottom_panel;
mod dialog;
mod map_canvas;
mod side_panel;

use self::bottom_panel::*;
use self::dialog::*;
use self::map_canvas::*;
use self::side_panel::*;
use crate::{Clear, Term};
use anyhow::{Context, Result};
use kartoffels_world::prelude::{BotId, Handle as WorldHandle};
use ratatui::layout::{Constraint, Layout};
use tokio::select;
use tokio_stream::StreamExt;

pub async fn run(term: &mut Term, handle: WorldHandle) -> Result<Outcome> {
    let mut snapshots = handle.listen().await?;

    let mut snapshot = snapshots
        .next()
        .await
        .context("lost connection to the world")?;

    let mut camera = snapshot.map.size().as_ivec2() / 2;
    let mut bot: Option<JoinedBot> = None;
    let mut dialog: Option<Dialog> = None;
    let mut paused = false;

    loop {
        if let Some(bot) = &bot {
            if bot.follow_with_camera {
                if let Some(bot) = snapshot.bots.alive.by_id(bot.id) {
                    camera = bot.pos;
                }
            }
        }

        let mut event = None;

        term.draw(|ui| {
            let [main_area, bottom_area] =
                Layout::vertical([Constraint::Fill(1), Constraint::Length(1)])
                    .areas(ui.area());

            let [map_area, side_area] = Layout::horizontal([
                Constraint::Fill(1),
                Constraint::Length(SidePanel::WIDTH),
            ])
            .areas(main_area);

            let enabled = dialog.is_none();

            Clear::render(ui);

            let bottom_panel_event = ui
                .clamp(bottom_area, |ui| {
                    BottomPanel::render(ui, paused, enabled)
                })
                .map(Event::BottomPanel);

            let side_panel_event = ui
                .clamp(side_area, |ui| {
                    SidePanel::render(ui, &snapshot, bot.as_ref(), enabled)
                })
                .map(Event::SidePanel);

            let map_canvas_event = ui
                .clamp(map_area, |ui| {
                    MapCanvas::render(ui, &snapshot, camera, paused, enabled)
                })
                .map(Event::MapCanvas);

            let dialog_event = dialog
                .as_mut()
                .and_then(|dialog| dialog.render(ui, &snapshot))
                .map(Event::Dialog);

            event = bottom_panel_event
                .or(side_panel_event)
                .or(map_canvas_event)
                .or(dialog_event);
        })
        .await?;

        snapshot = select! {
            _ = term.tick() => {
                continue;
            }

            snapshot = snapshots.next() => {
                snapshot.context("lost connection to the world")?
            },
        };
    }
}

// #[derive(Debug)]
// struct View {
//     camera: IVec2,
//     bot: Option<JoinedBot>,
//     dialog: Option<Dialog>,
//     paused: bool,
//     snapshot: Arc<WorldSnapshot>,
//     handle: WorldHandle,
// }

// impl View {
//     fn render(&mut self, area: Rect, buf: &mut Buffer) {
//         Clear.render(area, buf);
//     }

//     async fn handle_input(
//         &mut self,
//         mut event: InputEvent,
//         term: &Term,
//     ) -> Result<ControlFlow<Outcome, ()>> {
//         if let Some(dialog) = &mut self.dialog {
//             match dialog.handle(event, &self.snapshot) {
//                 Some(DialogEvent::Close) => {
//                     self.dialog = None;
//                 }

//                 Some(DialogEvent::JoinBot(id)) => {
//                     self.dialog = None;
//                     self.handle_join_bot(id);
//                 }

//                 Some(DialogEvent::UploadBot(src)) => {
//                     self.dialog = None;
//                     self.handle_upload_bot(src).await?;
//                 }

//                 Some(DialogEvent::OpenTutorial) => {
//                     return Ok(ControlFlow::Break(Outcome::OpenTutorial));
//                 }

//                 Some(DialogEvent::Throw(error)) => {
//                     self.dialog = Some(Dialog::Error(ErrorDialog { error }));
//                 }

//                 None => {
//                     //
//                 }
//             }

//             return Ok(ControlFlow::Continue(()));
//         }

//         event = match SidePanel::handle(self.bot.is_some(), event) {
//             SidePanelEvent::UploadBot => {
//                 self.dialog = Some(Dialog::UploadBot(Default::default()));

//                 return Ok(ControlFlow::Continue(()));
//             }

//             SidePanelEvent::JoinBot => {
//                 self.dialog = Some(Dialog::JoinBot(Default::default()));

//                 return Ok(ControlFlow::Continue(()));
//             }

//             SidePanelEvent::LeaveBot => {
//                 self.bot = None;

//                 return Ok(ControlFlow::Continue(()));
//             }

//             SidePanelEvent::Forward(event) => event,
//         };

//         event = match MapCanvas::handle(event, term) {
//             MapCanvasEvent::MoveCamera(delta) => {
//                 self.camera += delta;

//                 return Ok(ControlFlow::Continue(()));
//             }

//             MapCanvasEvent::Forward(event) => event,
//         };

//         match BottomPanel::handle(event, self.paused) {
//             Some(BottomPanelEvent::Quit) => {
//                 return Ok(ControlFlow::Break(Outcome::Quit));
//             }

//             Some(BottomPanelEvent::Help) => {
//                 self.dialog = Some(Dialog::Help(Default::default()));
//             }

//             Some(BottomPanelEvent::Pause) => {
//                 self.paused = !self.paused;
//             }

//             Some(BottomPanelEvent::ListBots) => {
//                 self.dialog = Some(Dialog::Bots(Default::default()));
//             }

//             None => (),
//         }

//         Ok(ControlFlow::Continue(()))
//     }

//     fn handle_snapshot(
//         &mut self,
//         snapshot: Option<Arc<WorldSnapshot>>,
//     ) -> Result<()> {
//         let snapshot = snapshot.context("lost connection to the world")?;

//         if !self.paused {
//             self.snapshot = snapshot;
//         }

//         Ok(())
//     }

//     async fn handle_upload_bot(&mut self, src: String) -> Result<()> {
//         let src = src.trim().replace('\n', "");

//         let src = match BASE64_STANDARD.decode(src) {
//             Ok(src) => src,
//             Err(err) => {
//                 self.dialog = Some(Dialog::Error(ErrorDialog {
//                     error: format!(
//                         "couldn't decode pasted content:\n\n{}",
//                         err
//                     ),
//                 }));

//                 return Ok(());
//             }
//         };

//         let id = match self.handle.create_bot(src, None, false).await {
//             Ok(id) => id,

//             Err(err) => {
//                 self.dialog = Some(Dialog::Error(ErrorDialog {
//                     error: err.to_string(),
//                 }));

//                 return Ok(());
//             }
//         };

//         self.handle_join_bot(id);

//         Ok(())
//     }

//     fn handle_join_bot(&mut self, id: BotId) {
//         self.bot = Some(JoinedBot {
//             id,
//             follow_with_camera: true,
//         });
//     }

//     async fn tick(&mut self) {
//         if let Some(dialog) = &mut self.dialog {
//             dialog.tick().await;
//         } else {
//             future::pending::<()>().await;
//         }
//     }

//     fn is_enabled(&self) -> bool {
//         self.dialog.is_none()
//     }
// }

#[derive(Debug)]
enum Event {
    BottomPanel(BottomPanelEvent),
    Dialog(DialogEvent),
    MapCanvas(MapCanvasEvent),
    SidePanel(SidePanelEvent),
}

#[derive(Debug)]
struct JoinedBot {
    id: BotId,
    follow_with_camera: bool,
}

#[derive(Debug)]
pub enum Outcome {
    OpenTutorial,
    Quit,
}
