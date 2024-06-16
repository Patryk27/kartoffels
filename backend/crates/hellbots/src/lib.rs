#![feature(array_chunks)]
#![feature(extract_if)]
#![feature(hash_raw_entry)]
#![feature(inline_const_pat)]
#![feature(let_chains)]
#![feature(try_blocks)]
#![allow(clippy::result_unit_err)]

mod bot;
mod bot_id;
mod bots;
mod id;
mod loop_timer;
mod map;
mod mode;
mod serde;
mod theme;
mod world;
mod world_config;
mod world_id;
mod world_name;

pub use self::bot::*;
pub use self::bot_id::*;
pub use self::bots::*;
use self::id::*;
pub use self::loop_timer::*;
pub use self::map::*;
pub use self::mode::*;
pub use self::theme::*;
pub use self::world::*;
pub use self::world_config::*;
pub use self::world_id::*;
pub use self::world_name::*;
