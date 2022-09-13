use crate::prelude::*;

pub fn cube_with_moves(moves: &str) -> Cube {
    Cube::solved().apply_all(Move::parse_sequence(moves).unwrap())
}
