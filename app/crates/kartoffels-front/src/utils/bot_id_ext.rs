use kartoffels_world::prelude as w;
use ratatui::palette::Hsl;
use ratatui::style::Color;

pub trait BotIdExt {
    fn color(&self) -> Color;
}

impl BotIdExt for w::BotId {
    fn color(&self) -> Color {
        let hue = (self.get().get() % 360) as f32;

        Color::from_hsl(Hsl::new(hue, 1.0, 0.5))
    }
}
