use super::DialogResponse;
use crate::Ui;

#[derive(Debug, Default)]
pub struct ConfigureWorldDialog {
    //
}

impl ConfigureWorldDialog {
    pub fn render(&mut self, _ui: &mut Ui) -> Option<DialogResponse> {
        None
    }
}