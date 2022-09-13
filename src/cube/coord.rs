use crate::prelude::*;

/// Kociemba-style coordinate cubes.

pub struct CoordCube {}

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

pub fn corner_orientation(cube: &Cube) -> u32 {
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

pub fn edge_orientation(cube: &Cube) -> u32 {
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
            edge_orientation(&Cube::solved().apply_all(moves)) < 2u32.pow(11)
        }
    }
}
