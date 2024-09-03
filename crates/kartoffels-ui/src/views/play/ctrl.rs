#[derive(Debug)]
pub enum Controller {
    Normal,
    Sandbox,
}

impl Controller {
    pub fn is_sandbox(&self) -> bool {
        matches!(self, Controller::Sandbox)
    }
}
