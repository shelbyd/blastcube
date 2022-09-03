use crate::cube::*;

pub struct Move {
    face: Face,
    amount: Amount,
}

pub enum Amount {
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

        let amount = match chars.next() {
            None => Amount::Single,
            Some('\'') => Amount::Reverse,
            Some('2') => Amount::Double,
            Some(c) => return Err(anyhow::anyhow!("Unrecognized amount {}", c)),
        };

        Ok(Move { face, amount })
    }
}
