#![feature(duration_constructors)]
#![feature(let_chains)]
#![feature(result_flattening)]
#![feature(try_blocks)]

mod session;
mod sessions;
mod world;
mod worlds;

pub use self::session::*;
use self::sessions::*;
pub use self::world::*;
use self::worlds::*;
use anyhow::{Context, Result, anyhow};
use derivative::Derivative;
use futures_util::FutureExt;
use kartoffels_world::prelude::{
    Clock, Config as WorldConfig, Handle as WorldHandle,
};
use std::ops::ControlFlow;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;
use tokio::{fs, select, task, time};
use tracing::{Instrument, Span, debug, error, info, warn};

#[derive(Debug)]
pub struct Store {
    tx: mpsc::Sender<(Span, Request)>,
    dir: Option<PathBuf>,
    testing: bool,
}

impl Store {
    const ERR: &'static str = "store has crashed";

    pub async fn open(dir: Option<&Path>, testing: bool) -> Result<Self> {
        info!("opening");

        let (tx, rx) = mpsc::channel(32);
        let dir = dir.map(Path::to_owned);

        task::spawn({
            let sessions = Sessions::new();
            let worlds = Worlds::new(dir.as_deref()).await?;

            let actor = Actor {
                dir: dir.clone(),
                testing,
                sessions,
                worlds,
                ongoing_save: None,
            };

            async move {
                let result = actor.start(rx).await;

                if let Err(err) = result {
                    error!("store died: {err:?}");
                }
            }
        });

        Ok(Self { tx, dir, testing })
    }

    pub fn dir(&self) -> &Path {
        self.dir.as_deref().unwrap()
    }

    pub fn testing(&self) -> bool {
        self.testing
    }

    pub async fn ping(&self) -> Result<()> {
        self.send(|tx| Request::Ping { tx }).await
    }

    pub async fn add_world(&self, handle: WorldHandle) -> Result<()> {
        self.send(|tx| Request::AddWorld { handle, tx }).await
    }

    pub async fn create_private_world(
        &self,
        config: WorldConfig,
    ) -> Result<World> {
        self.create_world(WorldVis::Private, config).await
    }

    pub async fn create_public_world(
        &self,
        config: WorldConfig,
    ) -> Result<World> {
        self.create_world(WorldVis::Public, config).await
    }

    async fn create_world(
        &self,
        vis: WorldVis,
        config: WorldConfig,
    ) -> Result<World> {
        self.send(|tx| Request::CreateWorld { vis, config, tx })
            .await?
    }

    pub async fn get_world(&self, id: WorldId) -> Result<World> {
        // TODO inefficient
        self.find_worlds(None)
            .await?
            .into_iter()
            .find(|world| world.id() == id)
            .with_context(|| format!("couldn't find world `{id}`"))
    }

    pub async fn find_worlds(
        &self,
        vis: impl Into<Option<WorldVis>>,
    ) -> Result<Vec<World>> {
        self.send(|tx| Request::FindWorlds {
            vis: vis.into(),
            tx,
        })
        .await
    }

    pub async fn rename_world(&self, id: WorldId, name: String) -> Result<()> {
        self.send(|tx| Request::RenameWorld { id, name, tx })
            .await?
    }

    pub async fn delete_world(&self, id: WorldId) -> Result<()> {
        self.send(|tx| Request::DeleteWorld { id, tx }).await?
    }

    pub fn world_config(&self, name: &str) -> WorldConfig {
        if self.testing() {
            WorldConfig {
                clock: Clock::manual(),
                events: true,
                name: name.into(),
                seed: Some(Default::default()),
                ..Default::default()
            }
        } else {
            WorldConfig {
                events: true,
                name: name.into(),
                ..Default::default()
            }
        }
    }

    pub async fn create_session(&self) -> Result<Session> {
        self.send(|tx| Request::CreateSession { tx }).await
    }

    pub async fn get_session(&self, id: SessionId) -> Result<Session> {
        self.find_sessions(id)
            .await?
            .into_iter()
            .next()
            .with_context(|| format!("couldn't find session `{id}`"))
    }

    pub async fn find_sessions(
        &self,
        id: impl Into<Option<SessionId>>,
    ) -> Result<Vec<Session>> {
        self.send(|tx| Request::FindSessions { id: id.into(), tx })
            .await
    }

    pub async fn close(&self) -> Result<()> {
        self.send(|tx| Request::Close { tx }).await?
    }

    async fn send<T>(
        &self,
        req: impl FnOnce(oneshot::Sender<T>) -> Request,
    ) -> Result<T> {
        let (tx, rx) = oneshot::channel();

        self.send_ex(req(tx)).await?;

        rx.await.context(Self::ERR)
    }

    async fn send_ex(&self, req: Request) -> Result<()> {
        self.tx
            .send((Span::current(), req))
            .await
            .map_err(|_| anyhow!("{}", Self::ERR))?;

        Ok(())
    }
}

#[derive(Debug)]
struct Actor {
    dir: Option<PathBuf>,
    testing: bool,
    sessions: Sessions,
    worlds: Worlds,
    ongoing_save: Option<JoinHandle<Result<()>>>,
}

impl Actor {
    async fn start(
        mut self,
        mut rx: mpsc::Receiver<(Span, Request)>,
    ) -> Result<()> {
        let mut ping = time::interval(Duration::from_mins(1));
        let mut save = time::interval(Duration::from_mins(15));

        loop {
            select! {
                request = rx.recv() => {
                    let (span, request) = request.context("store abandoned")?;

                    match self.handle(request).instrument(span).await {
                        ControlFlow::Continue(()) => {
                            continue;
                        }
                        ControlFlow::Break(()) => {
                            break Ok(());
                        }
                    }
                }

                _ = ping.tick() => {
                    self.ping().await?;
                }
                _ = save.tick() => {
                    self.save().await?;
                }

                _ = self.sessions.gc() => {}
                _ = self.worlds.gc() => {}
            }
        }
    }

    async fn handle(&mut self, request: Request) -> ControlFlow<(), ()> {
        debug!(?request, "handling request");

        match request {
            Request::Ping { tx } => {
                _ = tx.send(());
            }

            Request::AddWorld { handle, tx } => {
                self.worlds.add(self.testing, handle);

                _ = tx.send(());
            }

            Request::CreateWorld { vis, config, tx } => {
                _ = tx.send(
                    self.worlds
                        .create(self.testing, self.dir.as_deref(), vis, config)
                        .await,
                );
            }

            Request::FindWorlds { vis, tx } => {
                _ = tx.send(self.worlds.find(vis));
            }

            Request::RenameWorld { id, name, tx } => {
                _ = tx.send(self.worlds.rename(id, name));
            }

            Request::DeleteWorld { id, tx } => {
                _ = tx.send(self.worlds.delete(id).await);
            }

            Request::CreateSession { tx } => {
                _ = tx.send(self.sessions.create(self.testing));
            }

            Request::FindSessions { id, tx } => {
                _ = tx.send(self.sessions.find(id));
            }

            Request::Close { tx } => {
                _ = tx.send(self.shutdown().await);

                return ControlFlow::Break(());
            }
        }

        ControlFlow::Continue(())
    }

    async fn ping(&self) -> Result<()> {
        debug!("pinging worlds");

        for world in self.worlds.find(Some(WorldVis::Public)) {
            let result =
                time::timeout(Duration::from_millis(100), world.ping())
                    .await
                    .map_err(|_| anyhow!("timed out"))
                    .flatten();

            if let Err(err) = result {
                warn!(id=?world.id(), ?err, "couldn't ping world");

                return Err(anyhow!(
                    "couldn't ping world {}: {err:?}",
                    world.id()
                ));
            }
        }

        Ok(())
    }

    async fn save(&mut self) -> Result<()> {
        if let Some(mut task) = self.ongoing_save.take() {
            if let Some(result) = (&mut task).now_or_never() {
                result??;
            } else {
                warn!(
                    "previous save is still ongoing, waiting for it to finish"
                );
                warn!("this might indicate an I/O problem (e.g. slow disk)");

                task.await??;
            }
        }

        let worlds = self.worlds.find(Some(WorldVis::Public));

        if worlds.is_empty() {
            return Ok(());
        }

        self.ongoing_save = Some(task::spawn(async move {
            debug!("saving worlds");

            for world in worlds {
                if let Some(path) = world.path() {
                    Self::save_ex(&world, path, false).await.with_context(
                        || format!("couldn't save world `{}`", world.id()),
                    )?;
                }
            }

            Ok(())
        }));

        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        info!("shutting down");

        if let Some(mut task) = self.ongoing_save.take() {
            if let Some(result) = (&mut task).now_or_never() {
                result??;
            } else {
                info!("save is ongoing, waiting for it to finish");

                task.await??;
            }
        }

        debug!("saving worlds");

        for world in self.worlds.find(None) {
            if let Some(path) = world.path() {
                Self::save_ex(&world, path, true).await.with_context(|| {
                    format!("couldn't save world `{}`", world.id())
                })?;
            } else {
                world.shutdown().await?;
            }
        }

        Ok(())
    }

    async fn save_ex(world: &World, path: &Path, shutdown: bool) -> Result<()> {
        debug!(id=?world.id(), ?path, "saving world");

        let buffer = if shutdown {
            world.shutdown().await?
        } else {
            world.save().await?
        };

        let tmp_path = path.with_extension("world.new");

        fs::write(&tmp_path, buffer.into_vec())
            .await
            .with_context(|| {
                format!("couldn't write to `{}`", tmp_path.display(),)
            })?;

        fs::rename(&tmp_path, path).await.with_context(|| {
            format!(
                "couldn't rename `{}` into `{}`",
                tmp_path.display(),
                path.display(),
            )
        })?;

        Ok(())
    }
}

#[derive(Derivative)]
#[derivative(Debug)]
enum Request {
    Ping {
        #[derivative(Debug = "ignore")]
        tx: oneshot::Sender<()>,
    },

    AddWorld {
        handle: WorldHandle,
        #[derivative(Debug = "ignore")]
        tx: oneshot::Sender<()>,
    },

    CreateWorld {
        vis: WorldVis,
        config: WorldConfig,
        #[derivative(Debug = "ignore")]
        tx: oneshot::Sender<Result<World>>,
    },

    FindWorlds {
        vis: Option<WorldVis>,
        #[derivative(Debug = "ignore")]
        tx: oneshot::Sender<Vec<World>>,
    },

    RenameWorld {
        id: WorldId,
        name: String,
        #[derivative(Debug = "ignore")]
        tx: oneshot::Sender<Result<()>>,
    },

    DeleteWorld {
        id: WorldId,
        #[derivative(Debug = "ignore")]
        tx: oneshot::Sender<Result<()>>,
    },

    CreateSession {
        #[derivative(Debug = "ignore")]
        tx: oneshot::Sender<Session>,
    },

    FindSessions {
        id: Option<SessionId>,
        #[derivative(Debug = "ignore")]
        tx: oneshot::Sender<Vec<Session>>,
    },

    Close {
        #[derivative(Debug = "ignore")]
        tx: oneshot::Sender<Result<()>>,
    },
}
