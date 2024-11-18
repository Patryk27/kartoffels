use crate::{Object, ObjectId};
use ahash::AHashMap;
use glam::IVec2;
use rand::{Rng, RngCore};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Objects {
    objects: AHashMap<ObjectId, Object>,
    pos_to_id: AHashMap<IVec2, ObjectId>,
    id_to_pos: AHashMap<ObjectId, IVec2>,
}

impl Objects {
    pub fn create(
        &mut self,
        rng: &mut impl RngCore,
        obj: Object,
        pos: Option<IVec2>,
    ) -> ObjectId {
        let id = loop {
            let id = rng.gen();

            if !self.objects.contains_key(&id) {
                break id;
            }
        };

        self.add(id, obj, pos);

        id
    }

    pub fn add(&mut self, id: ObjectId, obj: Object, pos: Option<IVec2>) {
        self.objects.insert(id, obj);

        if let Some(pos) = pos {
            self.pos_to_id.insert(pos, id);
            self.id_to_pos.insert(id, pos);
        }
    }

    pub fn get(&self, id: ObjectId) -> Option<Object> {
        self.objects.get(&id).copied()
    }

    pub fn get_at(&self, pos: IVec2) -> Option<Object> {
        self.get(self.lookup_at(pos)?)
    }

    pub fn remove(&mut self, id: ObjectId) -> Option<Object> {
        let obj = self.objects.remove(&id)?;

        if let Some(pos) = self.id_to_pos.remove(&id) {
            self.pos_to_id.remove(&pos).unwrap();
        }

        Some(obj)
    }

    pub fn remove_at(&mut self, pos: IVec2) -> Option<(ObjectId, Object)> {
        let id = self.lookup_at(pos)?;
        let obj = self.remove(id).unwrap();

        Some((id, obj))
    }

    pub fn lookup_at(&self, pos: IVec2) -> Option<ObjectId> {
        self.pos_to_id.get(&pos).copied()
    }

    pub fn iter(&self) -> impl Iterator<Item = ObjectEntry> + '_ {
        self.objects.iter().map(|(id, obj)| ObjectEntry {
            id: *id,
            pos: self.id_to_pos.get(id).copied(),
            obj: *obj,
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ObjectEntry {
    pub id: ObjectId,
    pub obj: Object,
    pub pos: Option<IVec2>,
}
