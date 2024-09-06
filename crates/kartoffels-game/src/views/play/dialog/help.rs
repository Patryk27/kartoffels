use kartoffels_ui::Dialog;

pub type HelpDialog = Dialog<'static, HelpDialogResponse>;
pub type HelpDialogRef = &'static HelpDialog;

#[derive(Clone, Copy, Debug)]
pub enum HelpDialogResponse {
    Copy(&'static str),
    Close,
}
