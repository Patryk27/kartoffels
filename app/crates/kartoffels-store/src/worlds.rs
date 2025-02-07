use crate::WorldType;
use ahash::AHashMap;
use anyhow::{anyhow, Context, Result};
use arc_swap::ArcSwap;
use itertools::Itertools;
use kartoffels_utils::{ArcSwapExt, Id};
use kartoffels_world::prelude::{Config as WorldConfig, Handle as WorldHandle};
use std::collections::hash_map;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::fs;
use tracing::{debug, info, instrument};

#[derive(Debug)]
pub struct Worlds {
    entries: Arc<ArcSwap<AHashMap<Id, WorldEntry>>>,
    public_idx: ArcSwap<Vec<WorldHandle>>,
    test_next_id: AtomicU64,
}

impl Worlds {
    pub const MAX_WORLDS: usize = 128;

    pub async fn new(dir: Option<&Path>) -> Result<Self> {
        let entries = if let Some(dir) = dir {
            Self::load(dir).await?
        } else {
            Default::default()
        };

        let public_idx = build_public_idx(&entries);

        Ok(Self {
            entries: Arc::new(ArcSwap::from_pointee(entries)),
            public_idx: ArcSwap::from_pointee(public_idx),
            test_next_id: AtomicU64::new(1),
        })
    }

    #[instrument]
    async fn load(dir: &Path) -> Result<AHashMap<Id, WorldEntry>> {
        info!("loading worlds");

        let mut entries = AHashMap::new();
        let mut files = fs::read_dir(dir).await?;

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

                let handle = kartoffels_world::resume(id, &path)?;

                entries.insert(
                    id,
                    WorldEntry {
                        ty: WorldType::Public,
                        handle: Some(handle),
                    },
                );
            };

            result.with_context(|| {
                format!("couldn't load world: {}", path.display())
            })?;
        }

        Ok(entries)
    }

    pub fn create(
        &self,
        testing: bool,
        dir: Option<&Path>,
        ty: WorldType,
        config: WorldConfig,
    ) -> Result<WorldHandle> {
        debug!(?ty, ?config, "creating world");

        assert!(config.id.is_none());
        assert!(config.path.is_none());

        let id = self.create_alloc(testing, ty)?;
        let config = self.create_config(dir, ty, config, id);
        let handle = self.create_spawn(ty, config);

        if let WorldType::Public = ty {
            self.rebuild_public_idx();
        }

        info!(?id, "world created");

        Ok(handle)
    }

    fn create_alloc(&self, testing: bool, ty: WorldType) -> Result<Id> {
        let mut id = None;

        self.entries.try_rcu(|entries| {
            if entries.len() >= Self::MAX_WORLDS {
                return Err(anyhow!(
                    "ouch, the server is currently overloaded"
                ));
            }

            let mut entries = (**entries).clone();

            id = Some(loop {
                let id = if testing {
                    Id::new(self.test_next_id.fetch_add(1, Ordering::Relaxed))
                } else {
                    rand::random()
                };

                if let hash_map::Entry::Vacant(entry) = entries.entry(id) {
                    entry.insert(WorldEntry { ty, handle: None });

                    break id;
                }
            });

            Ok(entries)
        })?;

        Ok(id.unwrap())
    }

    fn create_config(
        &self,
        dir: Option<&Path>,
        ty: WorldType,
        mut config: WorldConfig,
        id: Id,
    ) -> WorldConfig {
        config.id = Some(id);

        if let WorldType::Public = ty {
            config.path = dir.map(|dir| path(dir, id));
        }

        config
    }

    fn create_spawn(&self, ty: WorldType, config: WorldConfig) -> WorldHandle {
        let id = config.id.unwrap();
        let handle = kartoffels_world::create(config);

        self.entries.rcu(|entries| {
            let mut entries = (**entries).clone();

            entries.get_mut(&id).unwrap().handle = Some(handle.clone());
            entries
        });

        match ty {
            WorldType::Public => handle,

            WorldType::Private => handle.on_last_drop({
                let entries = self.entries.clone();

                move || {
                    info!(?id, "world abandoned");

                    entries.rcu(|entries| {
                        let mut entries = (**entries).clone();

                        entries.remove(&id);
                        entries
                    });
                }
            }),
        }
    }

    #[instrument(skip(self))]
    pub async fn rename(&self, id: Id, name: String) -> Result<()> {
        debug!("renaming world");

        let entries = self.entries.load();

        let entry = entries
            .get(&id)
            .with_context(|| format!("couldn't find world `{id}`"))?;

        if let Some(handle) = &entry.handle {
            handle.rename(name).await?;

            if let WorldType::Public = entry.ty {
                self.rebuild_public_idx();
            }

            info!("world renamed");
        }

        Ok(())
    }

    #[instrument(skip(self, dir))]
    pub async fn delete(&self, dir: Option<&Path>, id: Id) -> Result<()> {
        debug!("deleting world");

        let entry = self.delete_remove(id)?;

        self.delete_cleanup(dir, id, entry).await?;

        info!("world deleted");

        Ok(())
    }

    fn delete_remove(&self, id: Id) -> Result<WorldEntry> {
        let mut entry = None;

        _ = self.public_idx.try_rcu(|entries| {
            if let Some((idx, _)) =
                entries.iter().find_position(|entry| entry.id() == id)
            {
                let mut entries = (**entries).clone();

                entries.remove(idx);

                Ok(entries)
            } else {
                // No need to update the index (not really an error, it's just
                // that we don't have a better-named ArcSwap combinator at hand)
                Err(())
            }
        });

        self.entries.try_rcu(|entries| -> Result<_> {
            let mut entries = (**entries).clone();

            entry = Some(
                entries
                    .remove(&id)
                    .with_context(|| format!("couldn't find world `{id}`"))?,
            );

            Ok(entries)
        })?;

        Ok(entry.unwrap())
    }

    async fn delete_cleanup(
        &self,
        dir: Option<&Path>,
        id: Id,
        entry: WorldEntry,
    ) -> Result<()> {
        if let Some(handle) = entry.handle {
            handle.shutdown().await?;
        }

        if let WorldType::Public = entry.ty
            && let Some(dir) = dir
        {
            let path = path(dir, id);

            fs::remove_file(&path).await.with_context(|| {
                format!("couldn't remove world's file `{}`", path.display())
            })?;
        }

        Ok(())
    }

    pub fn set(&self, handles: impl IntoIterator<Item = WorldHandle>) {
        let entries = handles
            .into_iter()
            .map(|handle| {
                let key = handle.id();

                let val = WorldEntry {
                    ty: WorldType::Public,
                    handle: Some(handle),
                };

                (key, val)
            })
            .collect();

        let public_idx = build_public_idx(&entries);

        self.entries.store(Arc::new(entries));
        self.public_idx.store(Arc::new(public_idx));
    }

    pub fn list(&self, ty: Option<WorldType>) -> Vec<(WorldType, WorldHandle)> {
        self.entries
            .load()
            .values()
            .filter(|entry| ty.map_or(true, |ty| ty == entry.ty))
            .filter_map(|entry| Some((entry.ty, entry.handle.clone()?)))
            .sorted_by(|(_, a), (_, b)| a.id().cmp(&b.id()))
            .collect()
    }

    pub fn public(&self) -> Arc<Vec<WorldHandle>> {
        self.public_idx.load_full()
    }

    pub fn first_private(&self) -> Option<WorldHandle> {
        self.entries
            .load()
            .values()
            .filter(|entry| matches!(entry.ty, WorldType::Private))
            .filter_map(|entry| entry.handle.clone())
            .next()
    }

    pub async fn shutdown(&self) -> Result<()> {
        for entry in self.entries.load().values() {
            if let WorldType::Public = entry.ty
                && let Some(handle) = &entry.handle
            {
                handle.shutdown().await?;
            }
        }

        Ok(())
    }

    fn rebuild_public_idx(&self) {
        let public_idx = build_public_idx(&self.entries.load());

        self.public_idx.store(Arc::new(public_idx));
    }
}

#[derive(Clone, Debug)]
struct WorldEntry {
    ty: WorldType,
    handle: Option<WorldHandle>,
}

fn path(dir: &Path, id: Id) -> PathBuf {
    dir.join(id.to_string()).with_extension("world")
}

fn build_public_idx(entries: &AHashMap<Id, WorldEntry>) -> Vec<WorldHandle> {
    entries
        .values()
        .filter(|entry| matches!(entry.ty, WorldType::Public))
        .filter_map(|entry| entry.handle.clone())
        .sorted_by(|a, b| a.name().as_str().cmp(b.name().as_str()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::array;

    #[tokio::test]
    async fn smoke() {
        let target = Worlds::new(None).await.unwrap();

        let [h1, h2, h3, h4] = array::from_fn(|idx| {
            let idx = idx + 1;

            let ty = if idx % 2 == 0 {
                WorldType::Public
            } else {
                WorldType::Private
            };

            let config = WorldConfig {
                name: format!("w{idx}"),
                ..Default::default()
            };

            target.create(true, None, ty, config).unwrap()
        });

        assert_eq!(h1.id(), Id::new(1));
        assert_eq!(h2.id(), Id::new(2));
        assert_eq!(h3.id(), Id::new(3));
        assert_eq!(h4.id(), Id::new(4));

        // ---

        let list = |ty: Option<WorldType>| -> Vec<(WorldType, Id)> {
            target
                .list(ty)
                .into_iter()
                .map(|(ty, handle)| (ty, handle.id()))
                .sorted_by_key(|(_, id)| *id)
                .collect()
        };

        assert_eq!(
            vec![
                (WorldType::Private, h1.id()),
                (WorldType::Public, h2.id()),
                (WorldType::Private, h3.id()),
                (WorldType::Public, h4.id()),
            ],
            list(None),
        );

        assert_eq!(
            vec![(WorldType::Private, h1.id()), (WorldType::Private, h3.id()),],
            list(Some(WorldType::Private)),
        );

        assert_eq!(
            vec![(WorldType::Public, h2.id()), (WorldType::Public, h4.id()),],
            list(Some(WorldType::Public)),
        );

        // ---

        assert_eq!(
            vec![h2.id(), h4.id()],
            target.public().iter().map(|h| h.id()).collect_vec()
        );

        // ---

        target.delete(None, h1.id()).await.unwrap();
        target.delete(None, h2.id()).await.unwrap();

        assert_eq!(
            vec![(WorldType::Private, h3.id()), (WorldType::Public, h4.id())],
            list(None),
        );

        assert_eq!(
            vec![h4.id()],
            target.public().iter().map(|h| h.id()).collect_vec()
        );

        // ---

        drop(h3);

        assert_eq!(vec![(WorldType::Public, h4.id())], list(None));
    }
}
