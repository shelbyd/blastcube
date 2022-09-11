use crate::prelude::*;

mod cubie;
mod facie;
mod surface;

pub use facie::Location;
pub use surface::Cube;

pub trait CubeLike: Sized + core::fmt::Debug + Eq {
    fn solved() -> Self;
    fn apply(self, move_: Move) -> Self;

    fn apply_all(self, moves: impl IntoIterator<Item = Move>) -> Self {
        moves.into_iter().fold(self, |cube, m| cube.apply(m))
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Debug, enum_iterator::Sequence)]
pub enum Face {
    Front,
    Back,
    Left,
    Right,
    Up,
    Down,
}

impl Face {
    pub fn same_axis(a: Face, b: Face) -> bool {
        if a == b {
            return true;
        }

        if a > b {
            return Face::same_axis(b, a);
        }

        match (a, b) {
            (Face::Front, Face::Back) => true,
            (Face::Left, Face::Right) => true,
            (Face::Up, Face::Down) => true,
            _ => false,
        }
    }
}

impl core::fmt::Display for Face {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Face::Up => "U",
                Face::Down => "D",
                Face::Front => "F",
                Face::Back => "B",
                Face::Left => "L",
                Face::Right => "R",
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solved_is_solved() {
        assert_eq!(Cube::solved(), Cube::solved());
    }

    #[test]
    fn single_move_is_not_solved() {
        assert_ne!(Cube::solved().apply("F2".parse().unwrap()), Cube::solved());
    }
}
