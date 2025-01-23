use ratatui::style::Color;
use std::time::Duration;

pub const BG: Color = Color::Rgb(9, 9, 9);
pub const FG: Color = Color::Rgb(240, 240, 240);

pub const BLUE: Color = Color::Rgb(0, 128, 255);
pub const DARKER_GRAY: Color = Color::Rgb(16, 16, 16);
pub const DARK_GRAY: Color = Color::Rgb(64, 64, 64);
pub const GRAY: Color = Color::Rgb(128, 128, 128);
pub const GREEN: Color = Color::Rgb(0, 255, 128);
pub const PINK: Color = Color::Rgb(255, 0, 128);
pub const RED: Color = Color::Rgb(255, 0, 0);
pub const WASHED_PINK: Color = Color::Rgb(200, 107, 133);
pub const WHITE: Color = Color::Rgb(255, 255, 255);
pub const YELLOW: Color = Color::Rgb(255, 212, 80);

pub const TIME_FORMAT: &str = "%H:%M:%S";
pub const DATETIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";
pub const DATETIME_LENGTH: u16 = 19;

pub const FRAME_TIME: Duration = Duration::from_millis(1000 / 30);
