use crate::cube::*;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct Move {
    pub face: Face,
    pub direction: Direction,
}

impl core::fmt::Debug for Move {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}{}", self.face, self.direction)
    }
}

#[derive(Clone, Copy, Debug, enum_iterator::Sequence, PartialEq, Eq, Hash)]
#[cfg_attr(test, derive(Arbitrary))]
pub enum Direction {
    Single,
    Double,
    Reverse,
}

impl Move {
    pub fn could_follow(&self, other: &Move) -> bool {
        if !Face::same_axis(self.face, other.face) {
            return true;
        }

        self.face > other.face
    }

    pub fn should_consider(seq: &[Move]) -> bool {
        seq.windows(2).all(|w| match w {
            [a, b] => b.could_follow(a),
            _ => unreachable!(),
        })
    }

    pub fn inverse_seq(seq: &[Move]) -> Vec<Move> {
        seq.into_iter().rev().map(|m| m.reverse()).collect()
    }

    pub fn parse_sequence(s: &str) -> anyhow::Result<Vec<Move>> {
        s.split(" ").map(|s| s.parse()).collect()
    }

    pub fn all() -> impl Iterator<Item = Move> {
        enum_iterator::all::<Face>().flat_map(|face| {
            enum_iterator::all::<Direction>().map(move |direction| Move { face, direction })
        })
    }

    pub fn reverse(&self) -> Move {
        Move {
            face: self.face,
            direction: self.direction.reverse(),
        }
    }
}

impl Direction {
    pub fn reverse(self) -> Direction {
        match self {
            Direction::Single => Direction::Reverse,
            Direction::Reverse => Direction::Single,
            Direction::Double => Direction::Double,
        }
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

impl core::fmt::Display for Move {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}{}", self.face, self.direction)
    }
}

impl core::fmt::Display for Direction {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Direction::Single => "",
                Direction::Reverse => "'",
                Direction::Double => "2",
            }
        )
    }
}
