use crate::{Render, Ui};

#[derive(Debug)]
pub struct WidgetList<'a, T> {
    items: Vec<T>,
    state: &'a mut WidgetListState,
}

impl<'a, T> WidgetList<'a, T> {
    pub fn new(
        items: impl IntoIterator<Item = T>,
        state: &'a mut WidgetListState,
    ) -> Self {
        Self {
            items: items.into_iter().collect(),
            state,
        }
    }
}

impl<E, T> Render<E> for WidgetList<'_, T>
where
    T: Render<E>,
{
    fn render(self, ui: &mut Ui<E>) {
        self.state.offset = self
            .state
            .offset
            .min(self.items.len() - (ui.area().height as usize));

        for (widget, area) in self
            .items
            .into_iter()
            .skip(self.state.offset)
            .zip(ui.area().rows())
        {
            ui.clamp(area, |ui| {
                widget.render(ui);
            });
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct WidgetListState {
    pub offset: usize,
    pub selected: Option<usize>,
}
