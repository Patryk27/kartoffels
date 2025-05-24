use crate::{World, WorldEntry, WorldId, WorldVis};
use anyhow::{anyhow, Context, Result};
use itertools::Itertools;
use kartoffels_utils::Id;
use kartoffels_world::prelude::{
    Config as WorldConfig, Handle as InnerWorldHandle, WorldBuffer,
};
use std::collections::{hash_map, HashMap};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::{fs, task};
use tracing::{debug, info, info_span, Span};

#[derive(Debug)]
pub(crate) struct Worlds {
    entries: HashMap<WorldId, Arc<WorldEntry>>,
    abandoned: (mpsc::Sender<WorldId>, mpsc::Receiver<WorldId>),
    test_id: u64,
}

impl Worlds {
    pub const MAX_WORLDS: usize = 128;

    pub async fn new(dir: Option<&Path>) -> Result<Self> {
        let entries = if let Some(dir) = dir {
            Self::load(dir).await?
        } else {
            Default::default()
        };

        Ok(Self {
            entries,
            abandoned: mpsc::channel(32),
            test_id: 0,
        })
    }

    async fn load(dir: &Path) -> Result<HashMap<WorldId, Arc<WorldEntry>>> {
        info!(?dir, "loading worlds");

        let mut files = fs::read_dir(dir).await?;
        let mut entries = HashMap::new();

        while let Some(file) = files.next_entry().await? {
            let path = file.path();

            let Some(stem) = path.file_stem().and_then(|stem| stem.to_str())
            else {
                continue;
            };

            let Some("world") = path.extension().and_then(|ext| ext.to_str())
            else {
                continue;
            };

            info!("loading: {}", path.display());

            let result: Result<()> = try {
                let id = stem
                    .parse()
                    .context("couldn't extract world id from path")?;

                let src = fs::read(&path).await?;
                let src = WorldBuffer::new(src);

                let handle = {
                    let span = info_span!("world", %id);
                    let _span = span.entered();

                    kartoffels_world::resume(src)?
                };

                let entry = Arc::new(WorldEntry {
                    vis: WorldVis::Public,
                    path: Some(path.clone()),
                    handle,
                });

                entries.insert(WorldId::new(id), entry);
            };

            result.with_context(|| {
                format!("couldn't load world: {}", path.display())
            })?;
        }

        Ok(entries)
    }

    pub fn add(&mut self, testing: bool, handle: InnerWorldHandle) {
        assert!(testing);

        let id = WorldId::new(Id::new({
            self.test_id += 1;
            self.test_id
        }));

        let entry = Arc::new(WorldEntry {
            vis: WorldVis::Public,
            path: None,
            handle,
        });

        self.entries.insert(id, entry);
    }

    pub async fn create(
        &mut self,
        testing: bool,
        dir: Option<&Path>,
        vis: WorldVis,
        config: WorldConfig,
    ) -> Result<World> {
        debug!(?vis, ?config, "creating world");

        if self.entries.len() >= Self::MAX_WORLDS {
            return Err(anyhow!("ouch, the server is currently overloaded"));
        }

        let (id, entry) = loop {
            let id = WorldId::new(if testing {
                Id::new({
                    self.test_id += 1;
                    self.test_id
                })
            } else {
                rand::random()
            });

            if let hash_map::Entry::Vacant(entry) = self.entries.entry(id) {
                break (id, entry);
            }
        };

        let path = match vis {
            WorldVis::Public => {
                dir.map(|dir| dir.join(id.to_string()).with_extension("world"))
            }
            WorldVis::Private => None,
        };

        let handle = {
            let span = Span::current();

            task::spawn_blocking(move || {
                let span = info_span!(parent: &span, "world", %id);
                let _span = span.entered();

                kartoffels_world::create(config)
            })
            .await?
        };

        let entry = entry.insert(Arc::new(WorldEntry {
            vis,
            path,
            handle: handle.clone(),
        }));

        info!(?id, "world created");

        let mut world = World::new(id, entry.clone());

        if world.vis() == WorldVis::Private {
            world = world.on_abandoned(self.abandoned.0.clone());
        }

        Ok(world)
    }

    pub fn find(&self, vis: Option<WorldVis>) -> Vec<World> {
        self.entries
            .iter()
            .filter(|(_, entry)| vis.is_none_or(|vis| vis == entry.vis))
            .map(|(id, entry)| World::new(*id, entry.clone()))
            .sorted_by_key(|world| world.id().get())
            .collect()
    }

    pub fn rename(&self, id: WorldId, name: String) -> Result<()> {
        debug!("renaming world");

        self.entries
            .get(&id)
            .with_context(|| format!("couldn't find world `{id}`"))?
            .handle
            .rename(name);

        info!("world renamed");

        Ok(())
    }

    pub async fn delete(&mut self, id: WorldId) -> Result<()> {
        debug!("deleting world");

        let world = self
            .entries
            .remove(&id)
            .with_context(|| format!("couldn't find world `{id}`"))?;

        world.handle.shutdown().await?;

        if let Some(path) = &world.path {
            fs::remove_file(path).await.with_context(|| {
                format!("couldn't remove world's file `{}`", path.display())
            })?;
        }

        Ok(())
    }

    pub async fn gc(&mut self) {
        if let Some(id) = self.abandoned.1.recv().await
            && let Some(entry) = self.entries.remove(&id)
        {
            task::spawn(async move {
                _ = entry.handle.shutdown().await;

                info!(?id, "world collected");
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn smoke() {
        let mut target = Worlds::new(None).await.unwrap();

        // ---

        let mut create = async |name: &str, vis: WorldVis| {
            let config = WorldConfig {
                name: name.into(),
                ..Default::default()
            };

            target.create(true, None, vis, config).await.unwrap()
        };

        let w1 = create("w1", WorldVis::Private).await;
        let w2 = create("w2", WorldVis::Public).await;
        let w3 = create("w3", WorldVis::Private).await;
        let w4 = create("w4", WorldVis::Public).await;

        assert_eq!(WorldId::new(Id::new(1)), w1.id());
        assert_eq!(WorldId::new(Id::new(2)), w2.id());
        assert_eq!(WorldId::new(Id::new(3)), w3.id());
        assert_eq!(WorldId::new(Id::new(4)), w4.id());

        // ---

        let list = |target: &Worlds, vis: Option<WorldVis>| -> Vec<_> {
            target
                .find(vis)
                .into_iter()
                .map(|world| world.id())
                .collect()
        };

        assert_eq!(
            vec![w1.id(), w2.id(), w3.id(), w4.id()],
            list(&target, None),
        );

        assert_eq!(
            vec![w1.id(), w3.id()],
            list(&target, Some(WorldVis::Private)),
        );

        assert_eq!(
            vec![w2.id(), w4.id()],
            list(&target, Some(WorldVis::Public)),
        );

        // ---

        target.delete(w1.id()).await.unwrap();
        target.delete(w2.id()).await.unwrap();

        assert_eq!(vec![w3.id(), w4.id()], list(&target, None));

        // ---

        drop(w3);
        target.gc().await;

        assert_eq!(vec![w4.id()], list(&target, None));
    }
}
