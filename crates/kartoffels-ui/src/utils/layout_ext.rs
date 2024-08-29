use ratatui::layout::{Constraint, Layout, Rect};

pub trait LayoutExt {
    fn dialog(width: Constraint, height: Constraint, area: Rect) -> Rect;
}

impl LayoutExt for Layout {
    fn dialog(width: Constraint, height: Constraint, area: Rect) -> Rect {
        let [_, area, _] = Layout::horizontal([
            Constraint::Fill(1),
            width,
            Constraint::Fill(1),
        ])
        .areas(area);

        let [_, area, _] = Layout::vertical([
            Constraint::Fill(1),
            height,
            Constraint::Fill(2),
        ])
        .areas(area);

        area
    }
}
