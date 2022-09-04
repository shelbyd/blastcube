use crate::cube::*;

#[derive(Clone, Copy)]
pub struct Move {
    pub face: Face,
    pub direction: Direction,
}

#[derive(Clone, Copy)]
pub enum Direction {
    Single,
    Double,
    Reverse,
}

impl Move {
    pub fn parse_sequence(s: &str) -> anyhow::Result<Vec<Move>> {
        s.split(" ").map(|s| s.parse()).collect()
    }
}

impl core::str::FromStr for Move {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Move> {
        let mut chars = s.chars();
        let face_char = match chars.next() {
            Some(c) => c,
            None => return Err(anyhow::anyhow!("No face for move")),
        };

        let face = match face_char {
            'F' | 'f' => Face::Front,
            'R' | 'r' => Face::Right,
            'U' | 'u' => Face::Up,
            'L' | 'l' => Face::Left,
            'B' | 'b' => Face::Back,
            'D' | 'd' => Face::Down,
            _ => return Err(anyhow::anyhow!("Unrecognized face {}", face_char)),
        };

        let direction = match chars.next() {
            None => Direction::Single,
            Some('\'') => Direction::Reverse,
            Some('2') => Direction::Double,
            Some(c) => return Err(anyhow::anyhow!("Unrecognized direction {}", c)),
        };

        Ok(Move { face, direction })
    }
}
