use ratatui::layout::{Constraint, Layout, Rect};

pub trait LayoutExt {
    fn dialog(width: u16, height: u16, area: Rect) -> Rect;
}

impl LayoutExt for Layout {
    fn dialog(width: u16, height: u16, area: Rect) -> Rect {
        let [_, area, _] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(width + 4),
            Constraint::Fill(1),
        ])
        .areas(area);

        let [_, area, _] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(height + 2),
            Constraint::Fill(2),
        ])
        .areas(area);

        area
    }
}
