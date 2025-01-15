use crate::storage::Header;
use crate::{
    Bots, Map, Metronome, Policy, Runs, SerializedWorld, Shutdown, Theme,
    WorldName, WorldPath, WorldRng,
};
use anyhow::Context;
use bevy_ecs::system::{Local, Res};
use maybe_owned::MaybeOwned;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};
use std::{fs, thread};
use tracing::{debug, warn, Span};

pub struct State {
    next_run_at: Instant,
    ongoing_save: Option<JoinHandle<()>>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            next_run_at: next_run_at(),
            ongoing_save: Default::default(),
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn save(
    mut state: Local<State>,
    bots: Res<Bots>,
    map: Res<Map>,
    name: Res<WorldName>,
    path: Option<Res<WorldPath>>,
    policy: Res<Policy>,
    rng: Res<WorldRng>,
    runs: Res<Runs>,
    shutdown: Option<Res<Shutdown>>,
    theme: Option<Res<Theme>>,
) {
    let Some(path) = path else {
        return;
    };

    if Instant::now() < state.next_run_at && shutdown.is_none() {
        return;
    }

    debug!("saving world");

    if let Some(handle) = state.ongoing_save.take() {
        if !handle.is_finished() {
            warn!("cannot save world: the previous save is still in progress");

            warn!(
                "to avoid overwriting the ongoing save, the engine will now \
                 block and wait for the previous save to complete",
            );

            warn!(
                "this might indicate a problem with I/O, e.g. an unresponsive \
                 disk",
            );
        }

        handle.join().expect("saving-thread crashed");
    }

    // ---

    let world = SerializedWorld {
        bots: MaybeOwned::Borrowed(&bots),
        map: MaybeOwned::Borrowed(&map),
        name: MaybeOwned::Borrowed(&name.0),
        policy: MaybeOwned::Borrowed(&policy),
        rng: MaybeOwned::Borrowed(&rng.0),
        runs: MaybeOwned::Borrowed(&runs),
        theme: theme.as_ref().map(|theme| MaybeOwned::Borrowed(&**theme)),
    };

    // Serializing directly into the file would be faster, but it also makes
    // the event loop potentially I/O bound, so let's first serialize into a
    // buffer and then move the I/O onto a thread pool.
    let (buffer, tt_ser) = Metronome::try_measure(|| {
        let mut buffer = Vec::new();

        Header::default()
            .write(&mut buffer)
            .context("couldn't write header")?;

        ciborium::into_writer(&world, &mut buffer)
            .context("couldn't write state")?;

        Ok(buffer)
    })
    .expect("couldn't save the world");

    let path = path.0.clone();
    let path_new = path.with_extension("world.new");
    let span = Span::current();

    let handle = thread::spawn(move || {
        let _span = span.entered();

        let (_, tt_io) = Metronome::try_measure(|| {
            fs::write(&path_new, &buffer).with_context(|| {
                format!("couldn't write: {}", path_new.display())
            })?;

            fs::rename(&path_new, &path).with_context(|| {
                format!(
                    "couldn't rename {} to {}",
                    path_new.display(),
                    path.display()
                )
            })?;

            Ok(())
        })
        .unwrap();

        debug!(?tt_ser, ?tt_io, "world saved");
    });

    // ---

    if shutdown.is_some() {
        handle.join().expect("saving-thread crashed");
    } else {
        state.ongoing_save = Some(handle);
        state.next_run_at = next_run_at();
    }
}

fn next_run_at() -> Instant {
    Instant::now() + Duration::from_mins(15)
}
