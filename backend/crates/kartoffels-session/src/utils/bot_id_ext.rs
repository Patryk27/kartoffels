use kartoffels_world::prelude::BotId;
use ratatui::style::Color;

pub trait BotIdExt {
    fn color(&self) -> Color;
}

impl BotIdExt for BotId {
    fn color(&self) -> Color {
        let hue = (self.get().get() % 360) as f64;

        Color::from_hsl(hue, 100.0, 50.0)
    }
}
