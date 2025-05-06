use crate::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RDir {
    Up,
    Right,
    Down,
    Left,
}

impl RDir {
    #[must_use]
    pub fn as_caret(&self) -> char {
        match self {
            RDir::Up => '^',
            RDir::Right => '>',
            RDir::Down => 'v',
            RDir::Left => '<',
        }
    }
}

impl ops::Mul<Dir> for RDir {
    type Output = Dir;

    fn mul(self, rhs: Dir) -> Self::Output {
        match self {
            RDir::Up => rhs,
            RDir::Right => rhs.turned_right(),
            RDir::Down => rhs.turned_back(),
            RDir::Left => rhs.turned_left(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(RDir::Up, '^')]
    #[test_case(RDir::Right, '>')]
    #[test_case(RDir::Down, 'v')]
    #[test_case(RDir::Left, '<')]
    fn as_caret(lhs: RDir, rhs: char) {
        assert_eq!(lhs.as_caret(), rhs);
    }
}
