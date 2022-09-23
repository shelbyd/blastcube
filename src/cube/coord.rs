use crate::prelude::*;

use std::collections::{BTreeMap, HashMap};

/// Kociemba-style coordinate cubes.

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CoordCube {
    pub raw: Cube,

    corner_orientation: u16,
    edge_orientation: u16,

    corner_position: u16,
    edge_position: u32,
}

impl From<Cube> for CoordCube {
    fn from(raw: Cube) -> Self {
        CoordCube {
            corner_orientation: corner_orientation(&raw),
            edge_orientation: edge_orientation(&raw),
            corner_position: corner_position(&raw),
            edge_position: edge_position(&raw),

            raw,
        }
    }
}

impl CoordCube {
    pub fn init_table() {
        lazy_static::initialize(&TRANSITION_TABLE);
    }

    pub fn apply(mut self, move_: Move) -> Self {
        self.raw = self.raw.apply(move_);

        self.corner_orientation = TRANSITION_TABLE
            .corner_orientation
            .get(self.corner_orientation, move_);
        self.edge_orientation = TRANSITION_TABLE
            .edge_orientation
            .get(self.edge_orientation, move_);
        self.corner_position = TRANSITION_TABLE
            .corner_position
            .get(self.corner_position, move_);
        self.edge_position = TRANSITION_TABLE
            .edge_position
            .get(self.edge_position, move_);

        self
    }

    pub fn corner_orientation(&self) -> u16 {
        self.corner_orientation
    }

    pub fn edge_orientation(&self) -> u16 {
        self.edge_orientation
    }

    pub fn corner_position(&self) -> u16 {
        self.corner_position
    }
}

lazy_static::lazy_static! {
    static ref TRANSITION_TABLE: TransitionTable = TransitionTable::init();
}

enum Axis {
    FB,
    UD,
    LR,
}

impl From<Face> for Axis {
    fn from(face: Face) -> Self {
        match face {
            Face::Up | Face::Down => Axis::UD,
            Face::Front | Face::Back => Axis::FB,
            Face::Left | Face::Right => Axis::LR,
        }
    }
}

fn corner_orientation(cube: &Cube) -> u16 {
    let mut count = 0;
    let value = Location::all().fold(0, |v, loc| {
        let value = match loc {
            Location::Center(_) | Location::Edge(_, _) => return v,

            // Skip the BRD cubie
            Location::Corner(
                Face::Back | Face::Right | Face::Down,
                Face::Back | Face::Right | Face::Down,
                Face::Back | Face::Right | Face::Down,
            ) => return v,

            Location::Corner(_, _, _) if !matches!(cube.get(loc), Face::Up | Face::Down) => {
                return v
            }
            Location::Corner(Face::Up | Face::Down, _, _) => 0,
            Location::Corner(Face::Front | Face::Back, _, _) => 1,
            Location::Corner(Face::Left | Face::Right, _, _) => 2,
        };
        count += 1;
        v * 3 + value
    });
    // We skip the BRD cubie
    assert_eq!(count, 7);
    value
}

fn corner_position(cube: &Cube) -> u16 {
    use Face::*;
    let ordered_cubes = Location::all()
        .filter_map(|loc| match loc {
            Location::Corner(major, minor, perp) if major < minor && minor < perp => {
                Some((major, minor, perp))
            }
            _ => None,
        })
        .map(|(major, minor, perp)| {
            let mut faces = [
                cube.get(Location::Corner(major, minor, perp)),
                cube.get(Location::Corner(minor, major, perp)),
                cube.get(Location::Corner(perp, major, minor)),
            ];
            faces.sort();
            (faces[0], faces[1], faces[2])
        })
        .map(|cubie| match cubie {
            (Front, Left, Up) => 0,
            (Front, Left, Down) => 1,
            (Front, Right, Up) => 2,
            (Front, Right, Down) => 3,
            (Back, Left, Up) => 4,
            (Back, Left, Down) => 5,
            (Back, Right, Up) => 6,
            (Back, Right, Down) => 7,
            _ => unreachable!(),
        })
        .collect::<Vec<_>>();
    assert_eq!(ordered_cubes.len(), 8);
    let bad_cubies_before = ordered_cubes
        .iter()
        .enumerate()
        .skip(1)
        .map(|(i, cube_id)| {
            ordered_cubes[0..i]
                .iter()
                .filter(|&other_cube| other_cube > cube_id)
                .count()
        });
    bad_cubies_before
        .enumerate()
        .fold(0, |v, (i, count)| v + factorial(i + 1) * count) as u16
}

fn factorial(n: usize) -> usize {
    match n {
        0 | 1 => 1,
        n => n * factorial(n - 1),
    }
}

fn edge_orientation(cube: &Cube) -> u16 {
    use Axis::*;

    let mut count = 0;
    let value = Location::all()
        .filter_map(|loc| match loc {
            Location::Center(_) | Location::Corner(_, _, _) => None,
            Location::Edge(Face::Back | Face::Right, Face::Back | Face::Right) => None,
            Location::Edge(ma, mi) => Some((ma, mi)),
        })
        .filter_map(|(major, minor)| {
            let this_face = cube.get(Location::Edge(major, minor));
            let other_face = cube.get(Location::Edge(minor, major));

            let v = match (
                this_face.into(),
                other_face.into(),
                major.into(),
                minor.into(),
            ) {
                (_, UD, _, _) => return None,

                (UD, _, UD, _) => true,
                (UD, _, FB, LR) => true,
                (UD, _, _, _) => false,

                (_, FB, _, _) => return None,

                (FB, LR, UD, _) => true,
                (FB, LR, FB, LR) => true,
                (FB, LR, _, _) => false,

                (LR, _, _, _) => return None,
            };
            Some(v)
        })
        .inspect(|_| count += 1)
        .fold(0, |v, is_good| v * 2 + if is_good { 0 } else { 1 });

    // We skip the BR edge.
    assert_eq!(count, 11);

    value
}

fn edge_position(cube: &Cube) -> u32 {
    use Face::*;
    let ordered_cubes = Location::all()
        .filter_map(|loc| match loc {
            Location::Edge(major, minor) if major < minor => Some((major, minor)),
            _ => None,
        })
        .map(|(major, minor)| {
            let mut faces = [
                cube.get(Location::Edge(major, minor)),
                cube.get(Location::Edge(minor, major)),
            ];
            faces.sort();
            (faces[0], faces[1])
        })
        .map(|cubie| match cubie {
            (Front, Left) => 0,
            (Front, Right) => 1,
            (Front, Up) => 2,
            (Front, Down) => 3,
            (Back, Left) => 4,
            (Back, Right) => 5,
            (Back, Up) => 6,
            (Back, Down) => 7,
            (Left, Up) => 8,
            (Left, Down) => 9,
            (Right, Up) => 10,
            (Right, Down) => 11,
            _ => unreachable!(),
        })
        .collect::<Vec<_>>();

    assert_eq!(ordered_cubes.len(), 12);
    let bad_cubies_before = ordered_cubes
        .iter()
        .enumerate()
        .skip(1)
        .map(|(i, cube_id)| {
            ordered_cubes[0..i]
                .iter()
                .filter(|&other_cube| other_cube > cube_id)
                .count()
        });

    bad_cubies_before
        .enumerate()
        .fold(0, |v, (i, count)| v + factorial(i + 1) * count) as u32
}

#[derive(Default)]
struct TransitionTable {
    corner_orientation: SingleTable<u16>,
    edge_orientation: SingleTable<u16>,
    corner_position: SingleTable<u16>,
    edge_position: SingleTable<u32>,
}

impl TransitionTable {
    fn init() -> Self {
        let mut table = TransitionTable::default();

        table
            .corner_orientation
            .populate_with("corner_orientation", corner_orientation);
        table
            .edge_orientation
            .populate_with("edge_orientation", edge_orientation);
        table
            .corner_position
            .populate_with("corner_position", corner_position);
        table
            .edge_position
            .populate_with("edge_position", edge_position);

        table
    }
}

#[derive(Default, Debug)]
struct SingleTable<T>(HashMap<Move, BTreeMap<T, T>>);

impl<T> SingleTable<T>
where
    T: core::hash::Hash + Eq + core::fmt::Debug + Copy + Ord,
{
    fn populate_with(&mut self, name: &str, f: impl Fn(&Cube) -> T) {
        use std::time::Instant;

        let start = std::time::Instant::now();
        log::info!("Populating transition table {}", name);

        let mut to_expand = BTreeMap::new();

        let solved = Cube::solved();
        to_expand.insert(f(&solved), solved);

        let log_every = Duration::from_millis(100);
        let mut last_log = Instant::now();
        while let Some((from_v, from)) = pop_front(&mut to_expand) {
            assert!(!self.has_outgoing(&from_v));

            if last_log.elapsed() >= log_every {
                last_log += log_every;
                log::info!("to_expand.len(): {}", to_expand.len());
                log::info!("     self.len(): {}", self.len());
            }

            for m in Move::all() {
                let to = from.clone().apply(m);
                let to_v = f(&to);

                self.insert(from_v, m, to_v);
                let should_expand =
                    !to_expand.contains_key(&to_v) && !self.has_outgoing(&to_v) && to_v != from_v;
                if should_expand {
                    let already = to_expand.insert(to_v, to);
                    assert_eq!(already, None);
                }
            }
        }

        log::info!(
            "Finished populating transition table {}, took {:?}, {} items",
            name,
            start.elapsed(),
            self.len(),
        );
    }

    fn get(&self, from: T, move_: Move) -> T {
        self.0[&move_][&from]
    }

    fn insert(&mut self, from: T, move_: Move, to: T) {
        if from == to {
            return;
        }

        let already = self.0.entry(move_).or_default().insert(from, to);
        assert_eq!(
            already, None,
            "Reinserted {:?} -> {:?} -> {:?}",
            from, move_, to
        );
    }

    fn len(&self) -> usize {
        self.0.values().map(|v| v.len()).sum()
    }

    fn has_outgoing(&self, t: &T) -> bool {
        self.0.values().any(|map| map.contains_key(t))
    }
}

fn pop_front<K: Ord + Clone, V>(map: &mut BTreeMap<K, V>) -> Option<(K, V)> {
    let key = map.keys().next()?.clone();
    map.remove_entry(&key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    mod corner_orientation {
        use super::*;

        #[test]
        fn solved_is_zero() {
            assert_eq!(corner_orientation(&Cube::solved()), 0);
        }

        #[test]
        fn right_turn_is_non_zero() {
            assert_ne!(
                corner_orientation(&Cube::solved().apply("R".parse().unwrap())),
                0
            );
        }

        #[test]
        fn left_is_not_right() {
            assert_ne!(
                corner_orientation(&Cube::solved().apply("R".parse().unwrap())),
                corner_orientation(&Cube::solved().apply("L".parse().unwrap())),
            );
        }

        #[test]
        fn double_right_is_zero() {
            assert_eq!(
                corner_orientation(&Cube::solved().apply("R2".parse().unwrap())),
                0
            );
        }

        #[test]
        fn double_back_not_double_front() {
            assert_ne!(
                corner_orientation(
                    &Cube::solved().apply_all(Move::parse_sequence("R' F2").unwrap())
                ),
                corner_orientation(
                    &Cube::solved().apply_all(Move::parse_sequence("R' B2").unwrap())
                ),
            );
        }

        #[test]
        fn opposite_twists() {
            let cw = cube_with_moves("L D2 L' F' D2 F U F' D2 F L D2 L' U'");
            let ccw = cube_with_moves("F' D2 F L D2 L' U L D2 L' F' D2 F U'");

            assert_ne!(corner_orientation(&cw), corner_orientation(&ccw));
        }
    }

    #[cfg(test)]
    mod corner_position {
        use super::*;

        #[test]
        fn solved_is_zero() {
            assert_eq!(corner_position(&Cube::solved()), 0);
        }

        #[test]
        fn twist_is_non_zero() {
            assert_ne!(corner_position(&cube_with_moves("F")), 0);
        }
    }

    #[cfg(test)]
    mod edge_orientation {
        use super::*;

        #[test]
        fn down_is_zero() {
            assert_eq!(
                edge_orientation(&Cube::solved().apply("D".parse().unwrap())),
                0
            );
        }

        #[test]
        fn front_is_not_zero() {
            assert_ne!(
                edge_orientation(&Cube::solved().apply("F".parse().unwrap())),
                0
            );
        }

        #[test]
        fn front_is_less_than_2048() {
            assert!(edge_orientation(&Cube::solved().apply("F".parse().unwrap())) < 2048);
        }

        #[test]
        fn sequence_less_than_2048() {
            assert!(
                edge_orientation(
                    &Cube::solved().apply_all(Move::parse_sequence("D' F B'").unwrap())
                ) < 2048
            );
        }

        #[quickcheck]
        fn always_less_than_2_pow_11(moves: Vec<Move>) -> bool {
            edge_orientation(&Cube::solved().apply_all(moves)) < 2_u16.pow(11)
        }
    }

    #[cfg(test)]
    mod edge_position {
        use super::*;

        #[test]
        fn solved_is_zero() {
            assert_eq!(edge_position(&Cube::solved()), 0);
        }

        #[test]
        fn twist_is_non_zero() {
            assert_ne!(edge_position(&cube_with_moves("F")), 0);
        }
    }
}
