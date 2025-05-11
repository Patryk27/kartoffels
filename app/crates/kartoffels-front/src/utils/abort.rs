use std::error::Error;
use std::fmt;
use termwiz::input::{KeyCode, Modifiers};

#[derive(Clone, Copy, Debug)]
pub struct Abort;

impl Abort {
    pub const BINDING: (KeyCode, Modifiers) =
        (KeyCode::Char('c'), Modifiers::CTRL);
}

impl fmt::Display for Abort {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "got C-c, bailing out")
    }
}

impl Error for Abort {
    //
}
