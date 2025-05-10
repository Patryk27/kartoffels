use super::AbsDir;
use std::ops;

/// Relative direction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RelDir {
    Up,
    Right,
    Down,
    Left,
}

impl RelDir {
    #[must_use]
    pub fn as_caret(&self) -> char {
        match self {
            RelDir::Up => '^',
            RelDir::Right => '>',
            RelDir::Down => 'v',
            RelDir::Left => '<',
        }
    }
}

impl ops::Mul<AbsDir> for RelDir {
    type Output = AbsDir;

    fn mul(self, rhs: AbsDir) -> Self::Output {
        match self {
            RelDir::Up => rhs,
            RelDir::Right => rhs.turned_right(),
            RelDir::Down => rhs.turned_back(),
            RelDir::Left => rhs.turned_left(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(RelDir::Up, '^')]
    #[test_case(RelDir::Right, '>')]
    #[test_case(RelDir::Down, 'v')]
    #[test_case(RelDir::Left, '<')]
    fn as_caret(lhs: RelDir, rhs: char) {
        assert_eq!(lhs.as_caret(), rhs);
    }
}
