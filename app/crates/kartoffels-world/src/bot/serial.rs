use crate::AliveBot;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::mem;
use std::sync::Arc;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BotSerial {
    curr: VecDeque<u32>,
    next: VecDeque<u32>,
    buffering: bool,

    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    snapshot: Option<Arc<VecDeque<u32>>>,
}

impl BotSerial {
    const CAPACITY: usize = 256;

    pub fn tick(&mut self) {
        // no-op
    }

    pub fn snapshot(&mut self) -> Arc<VecDeque<u32>> {
        self.snapshot
            .get_or_insert_with(|| Arc::new(self.curr.clone()))
            .clone()
    }

    pub fn mmio_load(&self, _addr: u32) -> Result<u32, ()> {
        Err(())
    }

    pub fn mmio_store(&mut self, addr: u32, val: u32) -> Result<(), ()> {
        match addr {
            AliveBot::MEM_SERIAL => {
                match val {
                    // serial_buffer()
                    0xffffff00 => {
                        self.buffering = true;
                    }

                    // serial_flush()
                    0xffffff01 => {
                        if self.buffering {
                            self.snapshot = None;
                            self.buffering = false;
                            self.curr.clear();

                            mem::swap(&mut self.curr, &mut self.next);
                        }
                    }

                    // serial_clear()
                    0xffffff02 => {
                        if self.buffering {
                            self.buffering = false;
                            self.next.clear();
                        }
                    }

                    val => {
                        let buf = if self.buffering {
                            &mut self.next
                        } else {
                            &mut self.curr
                        };

                        if buf.len() >= Self::CAPACITY {
                            buf.pop_front();
                        }

                        buf.push_back(val);

                        if !self.buffering {
                            self.snapshot = None;
                        }
                    }
                }

                Ok(())
            }

            _ => Err(()),
        }
    }
}
