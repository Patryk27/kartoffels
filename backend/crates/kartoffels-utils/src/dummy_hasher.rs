use std::hash::{BuildHasher, Hasher};

#[derive(Clone, Default)]
pub struct DummyHasher {
    data: u64,
}

impl BuildHasher for DummyHasher {
    type Hasher = Self;

    fn build_hasher(&self) -> Self::Hasher {
        Self::default()
    }
}

impl Hasher for DummyHasher {
    fn finish(&self) -> u64 {
        self.data
    }

    fn write(&mut self, _: &[u8]) {
        todo!();
    }

    fn write_u64(&mut self, i: u64) {
        self.data = i;
    }
}
