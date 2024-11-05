use std::error::Error;
use std::fmt;
use termwiz::input::{KeyCode, Modifiers};

#[derive(Clone, Copy, Debug)]
pub struct Abort {
    pub soft: bool,
}

impl Abort {
    pub const SOFT_BINDING: (KeyCode, Modifiers) =
        (KeyCode::Char('a'), Modifiers::CTRL);

    pub const HARD_BINDING: (KeyCode, Modifiers) =
        (KeyCode::Char('c'), Modifiers::CTRL);
}

impl fmt::Display for Abort {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.soft {
            write!(f, "got C-a, bailing out")
        } else {
            write!(f, "got C-c, bailing out")
        }
    }
}

impl Error for Abort {
    //
}
