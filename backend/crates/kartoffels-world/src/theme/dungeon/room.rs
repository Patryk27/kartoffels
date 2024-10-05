use super::{Corridor, CorridorDir};
use crate::{Map, TileBase};
use glam::{ivec2, IVec2};
use std::cmp;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Room {
    pub min: IVec2,
    pub max: IVec2,
}

impl Room {
    pub fn render(&self, map: &mut Map) {
        for y in self.min.y..=self.max.y {
            for x in self.min.x..=self.max.x {
                let tile = if x == self.min.x || x == self.max.x {
                    TileBase::WALL_V
                } else if y == self.min.y || y == self.max.y {
                    TileBase::WALL_H
                } else {
                    TileBase::FLOOR
                };

                map.set(ivec2(x, y), tile);
            }
        }
    }

    pub fn connect_with(&self, other: Self) -> Option<Corridor> {
        let start = ivec2(
            cmp::min(self.max.x - 1, other.max.x - 1),
            cmp::max(self.min.y + 1, other.min.y + 1),
        );

        let end = ivec2(
            cmp::max(self.min.x + 1, other.min.x + 1),
            cmp::min(self.max.y - 1, other.max.y - 1),
        );

        if start.distance_squared(end) >= 300 {
            return None;
        }

        if end.x > start.x && end.y > start.y {
            Some(Corridor {
                anchor: ivec2(start.x, (start.y + end.y) / 2),
                dir: CorridorDir::Horizontal,
                len: (end.x - start.x + 1) as u32,
            })
        } else if end.x < start.x && end.y < start.y {
            let (start, end) = (end, start);

            Some(Corridor {
                anchor: ivec2((start.x + end.x) / 2, start.y),
                dir: CorridorDir::Vertical,
                len: (end.y - start.y + 1) as u32,
            })
        } else {
            None
        }
    }

    pub fn collides_with(&self, other: Self, padding: i32) -> bool {
        (self.min.x - padding) <= other.max.x
            && (self.max.x + padding) >= other.min.x
            && (self.min.y - padding) <= other.max.y
            && (self.max.y + padding) >= other.min.y
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn room(min: IVec2, max: IVec2) -> Room {
        Room { min, max }
    }

    const fn corr_h(anchor: IVec2, len: u32) -> Corridor {
        Corridor {
            anchor,
            dir: CorridorDir::Horizontal,
            len,
        }
    }

    const fn corr_v(anchor: IVec2, len: u32) -> Corridor {
        Corridor {
            anchor,
            dir: CorridorDir::Vertical,
            len,
        }
    }

    mod connect_with {
        use super::*;
        use test_case::test_case;

        struct TestCase {
            lhs: Room,
            rhs: Room,
            expected: Option<Corridor>,
        }

        const TEST_1: TestCase = TestCase {
            lhs: room(ivec2(0, 0), ivec2(4, 4)),
            rhs: room(ivec2(8, 1), ivec2(12, 5)),
            expected: Some(corr_h(ivec2(3, 2), 7)),
        };

        const TEST_2: TestCase = TestCase {
            lhs: room(ivec2(0, 0), ivec2(4, 4)),
            rhs: room(ivec2(1, 8), ivec2(5, 12)),
            expected: Some(corr_v(ivec2(2, 3), 7)),
        };

        const TEST_3: TestCase = TestCase {
            lhs: room(ivec2(0, 0), ivec2(5, 3)),
            rhs: room(ivec2(8, 6), ivec2(10, 8)),
            expected: None,
        };

        #[test_case(TEST_1)]
        #[test_case(TEST_2)]
        #[test_case(TEST_3)]
        fn test(case: TestCase) {
            assert_eq!(case.lhs.connect_with(case.rhs), case.expected);
            assert_eq!(case.rhs.connect_with(case.lhs), case.expected);
        }
    }

    mod collides_with {
        use super::*;
        use test_case::test_case;

        struct TestCase {
            lhs: Room,
            rhs: Room,
            padding: i32,
            expected: bool,
        }

        const TEST_A1: TestCase = TestCase {
            lhs: room(ivec2(0, 0), ivec2(3, 3)),
            rhs: room(ivec2(2, 0), ivec2(6, 3)),
            padding: 0,
            expected: true,
        };

        const TEST_A2: TestCase = TestCase {
            lhs: room(ivec2(0, 0), ivec2(3, 3)),
            rhs: room(ivec2(3, 0), ivec2(7, 3)),
            padding: 0,
            expected: true,
        };

        const TEST_A3: TestCase = TestCase {
            lhs: room(ivec2(0, 0), ivec2(3, 3)),
            rhs: room(ivec2(4, 0), ivec2(8, 3)),
            padding: 0,
            expected: false,
        };

        const TEST_A4: TestCase = TestCase {
            lhs: room(ivec2(0, 0), ivec2(3, 3)),
            rhs: room(ivec2(5, 0), ivec2(9, 3)),
            padding: 0,
            expected: false,
        };

        const TEST_B1: TestCase = TestCase {
            lhs: room(ivec2(0, 0), ivec2(3, 3)),
            rhs: room(ivec2(0, 2), ivec2(3, 6)),
            padding: 0,
            expected: true,
        };

        const TEST_B2: TestCase = TestCase {
            lhs: room(ivec2(0, 0), ivec2(3, 3)),
            rhs: room(ivec2(0, 3), ivec2(3, 7)),
            padding: 0,
            expected: true,
        };

        const TEST_B3: TestCase = TestCase {
            lhs: room(ivec2(0, 0), ivec2(3, 3)),
            rhs: room(ivec2(0, 4), ivec2(3, 8)),
            padding: 0,
            expected: false,
        };

        const TEST_B4: TestCase = TestCase {
            lhs: room(ivec2(0, 0), ivec2(3, 3)),
            rhs: room(ivec2(0, 5), ivec2(3, 9)),
            padding: 0,
            expected: false,
        };

        const TEST_C1: TestCase = TestCase {
            lhs: room(ivec2(3, 3), ivec2(9, 9)),
            rhs: room(ivec2(1, 1), ivec2(5, 5)),
            padding: 0,
            expected: true,
        };

        const TEST_C2: TestCase = TestCase {
            lhs: room(ivec2(1, 1), ivec2(9, 9)),
            rhs: room(ivec2(3, 3), ivec2(6, 6)),
            padding: 0,
            expected: true,
        };

        const TEST_C3: TestCase = TestCase {
            lhs: room(ivec2(1, 1), ivec2(9, 9)),
            rhs: room(ivec2(1, 1), ivec2(9, 9)),
            padding: 0,
            expected: true,
        };

        #[test_case(TEST_A1)]
        #[test_case(TEST_A2)]
        #[test_case(TEST_A3)]
        #[test_case(TEST_A4)]
        #[test_case(TEST_B1)]
        #[test_case(TEST_B2)]
        #[test_case(TEST_B3)]
        #[test_case(TEST_B4)]
        #[test_case(TEST_C1)]
        #[test_case(TEST_C2)]
        #[test_case(TEST_C3)]
        fn test(case: TestCase) {
            assert_eq!(
                case.lhs.collides_with(case.rhs, case.padding),
                case.expected
            );

            assert_eq!(
                case.rhs.collides_with(case.lhs, case.padding),
                case.expected
            );
        }
    }
}
