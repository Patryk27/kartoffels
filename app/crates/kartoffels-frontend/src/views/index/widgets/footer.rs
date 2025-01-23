use kartoffels_store::Store;
use kartoffels_ui::{theme, Ui};
use ratatui::layout::Rect;
use ratatui::style::Stylize;
use ratatui::text::Span;
use std::sync::LazyLock;

pub struct Footer;

impl Footer {
    pub fn render<T>(store: &Store, ui: &mut Ui<T>) {
        let text = if store.testing() {
            "localhost"
        } else {
            &*FOOTER
        };

        let area = Rect {
            x: ui.area.width - text.len() as u16,
            y: ui.area.height - 1,
            width: text.len() as u16,
            height: 1,
        };

        ui.render_at(area, Span::raw(text).fg(theme::GRAY));
    }
}

static FOOTER: LazyLock<String> = LazyLock::new(|| {
    let url = "github:Patryk27/kartoffels";
    let rev = option_env!("KARTOFFELS_REV").unwrap_or("dirty");

    format!("{url}#{rev}")
});
