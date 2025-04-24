use crate::Dir;
use kartoffel::MEM_COMPASS;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BotCompass {
    dir: Option<Dir>,
    next_measurement_in: u32,
}

impl BotCompass {
    pub fn tick(&mut self, dir: Dir) {
        if let Some(time) = self.next_measurement_in.checked_sub(1) {
            self.next_measurement_in = time;
        } else {
            self.dir = Some(dir);
            self.next_measurement_in = 128_000;
        }
    }

    pub fn mmio_load(&mut self, addr: u32) -> Result<u32, ()> {
        match addr {
            MEM_COMPASS => Ok(match self.dir.take() {
                None => 0,
                Some(Dir::N) => 1,
                Some(Dir::E) => 2,
                Some(Dir::S) => 3,
                Some(Dir::W) => 4,
            }),

            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke() {
        let mut target = BotCompass::default();

        target.tick(Dir::N);

        assert_eq!(Ok(1), target.mmio_load(MEM_COMPASS));
        assert_eq!(Ok(0), target.mmio_load(MEM_COMPASS));

        // ---

        for _ in 0..128_000 {
            target.tick(Dir::N);
        }

        target.tick(Dir::E);

        assert_eq!(Ok(2), target.mmio_load(MEM_COMPASS));
        assert_eq!(Ok(0), target.mmio_load(MEM_COMPASS));

        // ---

        for _ in 0..128_000 {
            target.tick(Dir::N);
        }

        target.tick(Dir::S);

        assert_eq!(Ok(3), target.mmio_load(MEM_COMPASS));
        assert_eq!(Ok(0), target.mmio_load(MEM_COMPASS));

        // ---

        for _ in 0..128_000 {
            target.tick(Dir::N);
        }

        target.tick(Dir::W);

        assert_eq!(Ok(4), target.mmio_load(MEM_COMPASS));
        assert_eq!(Ok(0), target.mmio_load(MEM_COMPASS));
    }
}
