use super::AliveBotBody;
use kartoffel as api;
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

    pub fn snapshot(&mut self) -> Arc<VecDeque<u32>> {
        self.snapshot
            .get_or_insert_with(|| Arc::new(self.curr.clone()))
            .clone()
    }

    pub(super) fn store(
        bot: &mut AliveBotBody,
        addr: u32,
        val: u32,
    ) -> Result<(), ()> {
        match (addr, val) {
            (api::SERIAL_MEM, 0xffffff00) => {
                bot.serial.buffering = true;

                Ok(())
            }

            (api::SERIAL_MEM, 0xffffff01) => {
                if bot.serial.buffering {
                    bot.serial.snapshot = None;
                    bot.serial.buffering = false;
                    bot.serial.curr.clear();

                    mem::swap(&mut bot.serial.curr, &mut bot.serial.next);
                }

                Ok(())
            }

            (api::SERIAL_MEM, 0xffffff02) => {
                if bot.serial.buffering {
                    bot.serial.buffering = false;
                    bot.serial.next.clear();
                }

                Ok(())
            }

            (api::SERIAL_MEM, val) => {
                let buf = if bot.serial.buffering {
                    &mut bot.serial.next
                } else {
                    &mut bot.serial.curr
                };

                if buf.len() >= Self::CAPACITY {
                    buf.pop_front();
                }

                buf.push_back(val);

                if !bot.serial.buffering {
                    bot.serial.snapshot = None;
                }

                Ok(())
            }

            _ => Err(()),
        }
    }
}
