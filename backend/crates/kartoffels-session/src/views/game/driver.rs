use super::{Dialog, State};
use crate::DriverEvent;
use anyhow::Result;
use kartoffels_ui::Term;
use std::time::Instant;

impl DriverEvent {
    pub(super) async fn handle(
        self,
        state: &mut State,
        term: &mut Term,
    ) -> Result<()> {
        match self {
            DriverEvent::Join(handle) => {
                let mut snapshots = handle.snapshots();

                state.snapshot = snapshots.next().await?;
                state.snapshots = Some(snapshots);
                state.camera.move_to(state.snapshot.map().center());
                state.handle = Some(handle);
                state.bot = None;
            }

            DriverEvent::Pause => {
                state.pause().await?;
            }

            DriverEvent::Resume => {
                state.resume().await?;
            }

            DriverEvent::SetPerms(perms) => {
                state.perms = perms;
            }

            DriverEvent::UpdatePerms(f) => {
                f(&mut state.perms);
            }

            DriverEvent::OpenDialog(dialog) => {
                state.dialog = Some(Dialog::Custom(dialog));
            }

            DriverEvent::CloseDialog => {
                state.dialog = None;
            }

            DriverEvent::SetHelp(help) => {
                state.help = help;
            }

            DriverEvent::SetStatus(status) => {
                state.status = status.map(|status| (status, Instant::now()));
            }

            DriverEvent::CopyToClipboard(payload) => {
                term.copy_to_clipboard(payload).await?;
            }

            DriverEvent::GetSnapshotVersion(tx) => {
                _ = tx.send(state.snapshot.version());
            }
        }

        Ok(())
    }
}
