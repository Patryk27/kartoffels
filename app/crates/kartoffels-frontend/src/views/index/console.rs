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
use std::ops::ControlFlow;
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
    let mut sep = false;

    loop {
        let cmd = frame
            .tick(|ui| {
                ui.info_window(
                    ui.area.width,
                    ui.area.height,
                    Some(" console "),
                    |ui| {
                        ui.add(&mut term);
                    },
                );
            })
            .await?;

        if let Some(cmd) = cmd {
            info!(?cmd, "running command");

            if sep {
                _ = writeln!(term);
            } else {
                sep = true;
            }

            _ = writeln!(term, "; {cmd}");
            _ = writeln!(term);

            let cmd = match shellwords::split(&cmd) {
                Ok(cmd) => cmd,
                Err(err) => {
                    _ = writeln!(term, "{}", err.to_string().to_lowercase());
                    continue;
                }
            };

            let cmd = {
                let cmd = iter::once("kartoffels".into()).chain(cmd);

                match Cmd::try_parse_from(cmd) {
                    Ok(cmd) => cmd,

                    Err(err) => {
                        _ = write!(term, "{}", err.to_string().to_lowercase());
                        continue;
                    }
                }
            };

            match cmd.run(store, sess, &mut term).await {
                Ok(ControlFlow::Continue(_)) => {
                    //
                }
                Ok(ControlFlow::Break(_)) => {
                    return Ok(());
                }
                Err(err) => {
                    _ = writeln!(term, "{}", err.to_fmt_string());
                }
            }
        }
    }
}
