use crate::{Ui, theme};
use kartoffels_store::Store;
use ratatui::layout::Rect;
use ratatui::style::Stylize;
use ratatui::text::Span;
use std::sync::LazyLock;

static TEXT: LazyLock<String> = LazyLock::new(|| {
    let url = "github:Patryk27/kartoffels";
    let rev = option_env!("KARTOFFELS_REV").unwrap_or("dirty");

    format!("{url}#{rev}")
});

pub struct Footer;

impl Footer {
    pub fn render<T>(store: &Store, ui: &mut Ui<T>) {
        let text = if store.testing() { "localhost" } else { &*TEXT };

        let area = Rect {
            x: ui.area.width - text.len() as u16,
            y: ui.area.height - 1,
            width: text.len() as u16,
            height: 1,
        };

        ui.add_at(area, Span::raw(text).fg(theme::GRAY));
    }
}
