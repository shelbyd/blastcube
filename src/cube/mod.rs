use crate::prelude::*;

mod cubie;

pub use cubie::CubieCube as Cube;

pub trait CubeLike: Sized + core::fmt::Debug + Eq {
    fn solved() -> Self;
    fn apply(self, move_: Move) -> Self;

    fn apply_all(self, moves: impl IntoIterator<Item = Move>) -> Self {
        moves.into_iter().fold(self, |cube, m| cube.apply(m))
    }
}

pub enum Face {
    Front,
    Left,
    Right,
    Back,
    Up,
    Down,
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
