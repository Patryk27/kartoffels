use crate::{AliveBot, BotMmioContext};
use glam::{ivec2, IVec2};
use ringbuffer::{ConstGenericRingBuffer, RingBuffer};
use serde::de::{self, Visitor};
use serde::ser::SerializeStruct;
use serde::{Deserialize as AutoDeserialize, Serialize};

#[derive(Clone, Debug)]
pub struct BotBluetooth {
    cooldown: u32,
    messages: ConstGenericRingBuffer<[u8; 32], 5>,
    out_message: [u8; 32],
}

impl Serialize for BotBluetooth {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("BotBluetooth", 2)?;
        state.serialize_field("cooldown", &self.cooldown);
        let vec: Vec<[u8; 32]> = self.messages.to_vec();
        state.serialize_field("messages", &vec);
        state.serialize_field("out_message", &self.out_message);

        state.end()
    }
}
// We need to implement Deserialize for BotBluetooth since we use a ring buffer
impl<'de> AutoDeserialize<'de> for BotBluetooth {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[allow(non_camel_case_types)]
        #[derive(AutoDeserialize)]
        enum Field {
            cooldown,
            messages,
            out_message,
        }

        struct BotBluetoothVisitor;

        impl<'de> Visitor<'de> for BotBluetoothVisitor {
            type Value = BotBluetooth;

            fn expecting(
                &self,
                formatter: &mut std::fmt::Formatter,
            ) -> std::fmt::Result {
                formatter.write_str("struct BotBluetooth")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let cooldown = seq.next_element()?.ok_or_else(|| {
                    <serde_json::error::Error as de::Error>::invalid_length(
                        // this looks like the suggested de::Error doesn't have the correct type,
                        // TODO Check the version of serde, make sure we have the correct stuff
                        0, &self,
                    )
                });
                let messages: Result<Vec<[u8; 32]>, _> =
                    seq.next_element()?.ok_or_else(|| {
                        <serde_json::error::Error as de::Error>::invalid_length(
                            1, &self,
                        )
                    });
                let out_message = seq.next_element()?.ok_or_else(|| {
                    <serde_json::error::Error as de::Error>::invalid_length(
                        2, &self,
                    )
                });
                Ok(BotBluetooth {
                    cooldown: cooldown.unwrap(),
                    messages: ConstGenericRingBuffer::<[u8; 32], 5>::from(
                        messages.unwrap(),
                    ),
                    out_message: out_message.unwrap(),
                })
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut cooldown = None;
                let mut messages: Option<Vec<[u8; 32]>> = None;
                let mut out_message: Option<[u8; 32]> = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::cooldown => {
                            if cooldown.is_some() {
                                return Err(de::Error::duplicate_field(
                                    "cooldown",
                                ));
                            }
                            cooldown = Some(map.next_value()?);
                        }
                        Field::messages => {
                            if messages.is_some() {
                                return Err(de::Error::duplicate_field(
                                    "messages",
                                ));
                            }
                            messages = Some(map.next_value()?);
                        }
                        Field::out_message => {
                            if out_message.is_some() {
                                return Err(de::Error::duplicate_field(
                                    "messages",
                                ));
                            }
                            out_message = Some(map.next_value()?);
                        }
                    };
                }
                let cooldown = cooldown
                    .ok_or_else(|| {
                        <serde_json::error::Error as de::Error>::missing_field(
                            "cooldown",
                        )
                    })
                    .unwrap();
                let messages_holder: ConstGenericRingBuffer<[u8; 32], 5> =
                    ConstGenericRingBuffer::<[u8; 32], 5>::from(
                        messages
                            .ok_or_else(|| <serde_json::error::Error as de::Error>::missing_field("messages"))
                            .unwrap(),
                    );
                let out_message = out_message
                    .ok_or_else(|| {
                        <serde_json::error::Error as de::Error>::missing_field(
                            "out_message",
                        )
                    })
                    .unwrap();
                Ok(BotBluetooth {
                    cooldown,
                    messages: messages_holder,
                    out_message,
                })
            }
        }
        const FIELDS: &[&str] = &["cooldown", "messages", "out_message"];
        deserializer.deserialize_struct(
            "BotBluetooth",
            FIELDS,
            BotBluetoothVisitor,
        )
    }
}

impl BotBluetooth {
    pub fn tick(&mut self) {
        self.cooldown = self.cooldown.saturating_sub(1);
    }

    pub fn mmio_load(&self, addr: u32) -> Result<u32, ()> {
        match addr {
            AliveBot::MEM_BLUETOOTH => Ok((self.cooldown == 0) as u32),
            addr if addr >= AliveBot::MEM_BLUETOOTH + 4 => {
                let idx = addr - AliveBot::MEM_BLUETOOTH + 4;
                // we need to think of a way of reading from the message buffer here
                // current plan, actually return a [u8;4] constructed as [extra_info,buffer_index_of_fist_value,first_value,second_value]
                // the extra_info would be stuff like "is this an actual value from a buffer here" "is the message buffer empty"
                // the buffer_index_of_first_value would give the reader an idea of where they were in the message read operation, just for double checking means
                todo!();
            }
            _ => Err(()),
        }
    }

    pub fn recieve_message(self, message: [u8; 32]) -> Result<(), ()> {
        if self.messages.is_full() {
            return Err(());
        }
        self.messages.push(message);
        Ok(())
    }

    pub fn is_buffer_full(&mut self) -> bool {
        self.messages.is_full()
    }

    pub fn mmio_store(
        &mut self,
        ctxt: &mut BotMmioContext,
        addr: u32,
        val: u32,
    ) -> Result<(), ()> {
        match (addr, val.to_le_bytes()) {
            (AliveBot::MEM_BLUETOOTH, [0x01, range, _, _])
                if let Some(range) = BotBluetoothRange::new(range) =>
            {
                // the bot has been told to send a bluetooth message
                if self.cooldown == 0 {
                    self.send_message(ctxt, range)
                }
                Ok(())
            }
            (AliveBot::MEM_BLUETOOTH, [0x02, index, v, _]) if index < 32 => {
                // we write to the out_message
                self.out_message[index as usize] = v;
                Ok(())
            }
            (AliveBot::MEM_BLUETOOTH, [0x03, _, _, _]) => {
                self.out_message = [0; 32];
                Ok(())
            }
            // TODO: Maybe add a feature for checking if a bot has an open spot in their buffer
            _ => Err(()),
        }
    }

    fn send_message(
        &mut self,
        ctxt: &mut BotMmioContext,
        range: BotBluetoothRange,
    ) {
        for y in 0..range.len() {
            for x in 0..range.len() {
                let pos = {
                    let offset = ivec2(x as i32, y as i32)
                        - IVec2::splat(range.len() as i32) / 2;
                    ctxt.pos + ctxt.dir.as_vec().rotate(offset.perp())
                };

                if let Some(bot_id) = ctxt.bots.lookup_at(pos) {
                    let bot = ctxt.bots.get(bot_id).unwrap(); // the bot might die inbetween these instructions??
                    bot.bluetooth.recieve_message(self.out_message);
                }
            }
        }
        self.cooldown = range.cooldown(ctxt);
    }
}

impl Default for BotBluetooth {
    fn default() -> Self {
        Self {
            cooldown: 0,
            messages: ConstGenericRingBuffer::<[u8; 32], 5>::new(),
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, AutoDeserialize)]
enum BotBluetoothRange {
    D3 = 3,
    D5 = 5,
    D7 = 7,
    D9 = 9,
}

impl BotBluetoothRange {
    fn new(r: u8) -> Option<Self> {
        match r {
            3 => Some(Self::D3),
            5 => Some(Self::D5),
            7 => Some(Self::D7),
            9 => Some(Self::D9),
            _ => None,
        }
    }
    fn len(&self) -> u32 {
        *self as u32
    }
    fn cooldown(&self, ctxt: &mut BotMmioContext) -> u32 {
        match self {
            BotBluetoothRange::D3 => ctxt.cooldown(20_000, 10),
            BotBluetoothRange::D5 => ctxt.cooldown(23_000, 15),
            BotBluetoothRange::D7 => ctxt.cooldown(28_000, 25),
            BotBluetoothRange::D9 => ctxt.cooldown(35_000, 30),
        }
    }
}
// TODO TESTS
