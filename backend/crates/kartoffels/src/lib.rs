#![feature(array_chunks)]
#![feature(extract_if)]
#![feature(inline_const_pat)]
#![feature(let_chains)]
#![feature(try_blocks)]
#![allow(clippy::result_unit_err)]

mod bot;
mod bot_id;
mod bots;
mod map;
mod mode;
mod policy;
mod serde;
mod store;
mod theme;
mod utils;
mod world;
mod world_config;
mod world_id;
mod world_name;

pub use self::bot::*;
pub use self::bot_id::*;
pub use self::bots::*;
pub use self::map::*;
pub use self::mode::*;
pub use self::policy::*;
pub use self::store::*;
pub use self::theme::*;
pub use self::utils::*;
pub use self::world::*;
pub use self::world_config::*;
pub use self::world_id::*;
pub use self::world_name::*;
