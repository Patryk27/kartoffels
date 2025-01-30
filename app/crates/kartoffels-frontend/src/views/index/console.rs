mod cmds;
mod login;

use self::cmds::*;
use crate::Background;
use anyhow::Result;
use clap::Parser;
use kartoffels_store::{Session, Store};
use kartoffels_ui::{Frame, Term};
use kartoffels_utils::ErrorExt;
use std::fmt::Write;
use std::iter;
use tracing::{debug, info};

pub async fn run(
    store: &Store,
    sess: &Session,
    frame: &mut Frame,
    bg: &Background,
) -> Result<()> {
    debug!("run()");

    if sess.with(|sess| !sess.is_admin()) {
        match login::run(store, frame, bg).await? {
            login::Event::LoggedIn => {
                sess.with(|sess| {
                    sess.make_admin();
                });
            }

            login::Event::GoBack => {
                return Ok(());
            }
        }
    }

    let mut term = Term::default();

    loop {
        let cmd = frame
            .update(|ui| {
                ui.add(bg);

                let width = ui.area.width - 8;
                let height = ui.area.height - 4;
                let title = Some(" console ");

                ui.info_window(width, height, title, |ui| {
                    ui.add(&mut term);
                });
            })
            .await?;

        if let Some(cmd) = cmd {
            info!(?cmd, "running command");

            _ = writeln!(term, "; ---");
            _ = writeln!(term, "; {cmd}");
            _ = writeln!(term);

            let cmd = match shellwords::split(&cmd) {
                Ok(cmd) => cmd,
                Err(err) => {
                    _ = writeln!(term, "{err}");
                    continue;
                }
            };

            let cmd = {
                let cmd = iter::once("kartoffels".into()).chain(cmd);

                match Cmd::try_parse_from(cmd) {
                    Ok(cmd) => cmd,

                    Err(err) => {
                        _ = writeln!(term, "{err}");
                        continue;
                    }
                }
            };

            if let Err(err) = cmd.run(store, sess, &mut term) {
                _ = writeln!(term, "{}", err.to_fmt_string());
            }
        }
    }
}
