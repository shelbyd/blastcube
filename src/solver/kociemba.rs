use crate::prelude::*;

use core::{cmp::Ordering, hash::Hash};
use std::{collections::HashMap, sync::mpsc::channel, sync::Arc};

pub struct Kociemba<E: Evaluator> {
    challenge: Challenge<E>,

    to_domino: Phase,
    post_domino: Phase,
}

impl<E: Evaluator> Solver<E> for Kociemba<E> {
    fn init(challenge: Challenge<E>) -> Self {
        Kociemba {
            to_domino: {
                let moves = Move::all().collect::<Vec<_>>();
                let heuristics: Vec<Box<dyn Heuristic>> = vec![
                    Box::new(HeuristicTable::init(
                        "corner_orientation",
                        corner_orientation,
                        &moves,
                        &challenge.evaluator,
                        None,
                    )),
                    Box::new(HeuristicTable::init(
                        "edge_orientation",
                        edge_orientation,
                        &moves,
                        &challenge.evaluator,
                        None,
                    )),
                ];
                Phase::init(moves, is_domino_cube, heuristics)
            },
            post_domino: {
                let moves = domino_moves().collect::<Vec<_>>();
                Phase::init(moves, |c| *c == Cube::solved(), vec![])
            },

            challenge,
        }
    }

    fn solve(self: &Arc<Self>, cube: Cube) -> Box<dyn Iterator<Item = Move>> {
        let (tx, rx) = channel();

        let this = Arc::clone(self);
        let before_spawn = std::time::Instant::now();
        std::thread::spawn(move || {
            log::info!("Took {:?} to spawn worker thread", before_spawn.elapsed());
            let to_domino = this.solve_to(&cube, &this.to_domino, Vec::new());
            let domino_len = to_domino.len();
            for m in &to_domino {
                tx.send(*m).unwrap();
            }
            log::info!("Domino path: {:?}", to_domino);

            let solution = this.solve_to(&cube, &this.post_domino, to_domino);
            for m in &solution[domino_len..] {
                tx.send(*m).unwrap();
            }
        });

        Box::new(rx.into_iter())
    }
}

impl<E: Evaluator> Kociemba<E> {
    fn solve_to(&self, cube: &Cube, phase: &Phase, mut prefix: Vec<Move>) -> Vec<Move> {
        let cube = cube.clone().apply_all(prefix.clone());

        let mut best_time = self.challenge.evaluator.eval(&prefix);
        loop {
            log::info!("Searching <= {:?}", best_time);
            match self.find_solution(best_time, &cube, &mut prefix, phase) {
                Search::Found(moves) => return moves,
                Search::NotFound(next_best_time) => {
                    best_time = next_best_time;
                }
            }
        }
    }

    fn find_solution(
        &self,
        max_time: Duration,
        cube: &Cube,
        move_stack: &mut Vec<Move>,
        phase: &Phase,
    ) -> Search {
        let this_time = self.challenge.evaluator.eval(move_stack) + phase.min_time(cube);
        if this_time > max_time {
            return Search::NotFound(this_time);
        }

        if phase.is_finished(cube) {
            return Search::Found(move_stack.clone());
        }

        let last_move = move_stack.last().cloned();
        phase
            .allowed_moves
            .iter()
            .filter(|move_| match last_move {
                None => true,
                Some(m) => move_.could_follow(&m),
            })
            .fold(Search::NotFound(Duration::MAX), |best, &move_| {
                move_stack.push(move_);
                let cube = cube.clone().apply(move_);
                let sub = self.find_solution(max_time, &cube, move_stack, phase);
                move_stack.pop();

                match (best, sub) {
                    (Search::NotFound(a), Search::NotFound(b)) => {
                        Search::NotFound(core::cmp::min(a, b))
                    }
                    (Search::Found(m), Search::NotFound(_)) => Search::Found(m),
                    (Search::NotFound(_), Search::Found(m)) => Search::Found(m),
                    (Search::Found(a), Search::Found(b)) => {
                        let a_time = self.challenge.evaluator.eval(&a);
                        let b_time = self.challenge.evaluator.eval(&b);
                        Search::Found(match a_time.cmp(&b_time) {
                            Ordering::Less | Ordering::Equal => a,
                            Ordering::Greater => b,
                        })
                    }
                }
            })
            .into()
    }
}

enum Search {
    NotFound(Duration),
    Found(Vec<Move>),
}

fn domino_moves() -> impl Iterator<Item = Move> {
    Move::all().filter(is_domino_move)
}

fn is_domino_move(m: &Move) -> bool {
    match (m.face, m.direction) {
        (Face::Up | Face::Down, _) => true,
        (_, Direction::Double) => true,
        _ => false,
    }
}

fn is_domino_cube(cube: &Cube) -> bool {
    use Face::*;

    Location::all().all(|l| match (l, cube.get(l)) {
        (Location::Center(_), _) => true,

        (Location::Edge(Up | Down, _), Up | Down) => true,
        (Location::Corner(Up | Down, _, _), Up | Down) => true,

        (Location::Edge(Front | Back, Left | Right), Front | Back) => true,

        (Location::Edge(Front | Back, _), _) => true,
        (Location::Edge(Left | Right, _), _) => true,
        (Location::Corner(Front | Back | Left | Right, _, _), _) => true,

        _ => false,
    })
}

struct Phase {
    allowed_moves: Vec<Move>,
    finished_when: fn(&Cube) -> bool,
    heuristics: Vec<Box<dyn Heuristic>>,
}

impl Phase {
    fn init(
        allowed_moves: impl IntoIterator<Item = Move>,
        finished_when: fn(&Cube) -> bool,
        heuristics: Vec<Box<dyn Heuristic>>,
    ) -> Self {
        Self {
            allowed_moves: allowed_moves.into_iter().collect(),
            finished_when,
            heuristics,
        }
    }

    fn min_time(&self, cube: &Cube) -> Duration {
        self.heuristics
            .iter()
            .map(|h| h.min_time(cube))
            .max()
            .unwrap_or_default()
    }

    fn is_finished(&self, cube: &Cube) -> bool {
        (self.finished_when)(cube)
    }
}

trait Heuristic: Sync + Send {
    fn min_time(&self, cube: &Cube) -> Duration;
}

struct HeuristicTable<T: Eq + Hash> {
    #[allow(dead_code)]
    name: String,
    exhaustive: bool,

    map: HashMap<T, Duration>,
    simplifier: fn(&Cube) -> T,
}

impl<T: Eq + Hash + core::fmt::Debug> HeuristicTable<T> {
    fn init(
        name: &str,
        simplifier: fn(&Cube) -> T,
        allowed_moves: &[Move],
        evaluator: &impl Evaluator,
        max_setup: Option<Duration>,
    ) -> Self {
        let mut result = Self {
            name: name.to_string(),
            exhaustive: max_setup.is_none(),

            simplifier,
            map: HashMap::default(),
        };

        let start = std::time::Instant::now();
        for depth in 0..21 {
            log::info!(
                "{}: Expanding to depth: {}, {} items",
                result.name,
                depth,
                result.map.len()
            );
            let should_break = match max_setup {
                Some(max) if start.elapsed() >= max => true,
                _ => !result.expand_to_depth(depth, &mut Vec::new(), evaluator, allowed_moves),
            };
            if should_break {
                log::info!(
                    "{}: Finished expanding at depth {}, {} items, took {:?}",
                    result.name,
                    depth,
                    result.map.len(),
                    start.elapsed(),
                );
                break;
            }
        }

        result
    }

    fn expand_to_depth(
        &mut self,
        depth: usize,
        move_stack: &mut Vec<Move>,
        evaluator: &impl Evaluator,
        allowed_moves: &[Move],
    ) -> bool {
        let inv = Move::inverse_seq(move_stack);
        if !Move::should_consider(&inv) {
            return false;
        }

        let value = (self.simplifier)(&Cube::solved().apply_all(move_stack.clone()));
        let time = evaluator.min_time(&inv);

        let already = self.map.get(&value);
        match (depth, already) {
            (0, None) => {
                self.map.insert(value, time);
                true
            }
            (0, Some(t)) if time < *t => {
                self.map.insert(value, time);
                true
            }
            (0, Some(_)) => false,

            (_, Some(t)) if *t < time => false,
            (_, Some(_)) => allowed_moves.iter().fold(false, |any, move_| {
                move_stack.push(*move_);
                let result = self.expand_to_depth(depth - 1, move_stack, evaluator, allowed_moves);
                move_stack.pop();
                any || result
            }),
            (_, None) => unreachable!(),
        }
    }

    #[cfg(test)]
    fn has(&self, cube: &Cube) -> bool {
        let simplified = (self.simplifier)(cube);
        self.map.contains_key(&simplified)
    }
}

impl<T> Heuristic for HeuristicTable<T>
where
    T: Eq + Hash + Sync + Send + core::fmt::Debug,
{
    fn min_time(&self, cube: &Cube) -> Duration {
        let value = (self.simplifier)(cube);
        if let Some(d) = self.map.get(&value) {
            return *d;
        }

        if self.exhaustive {
            panic!(
                "{}: missing value ({:?}) for cube\n{}",
                self.name, value, cube
            );
        }
        Duration::default()
    }
}

fn corner_orientation(cube: &Cube) -> u32 {
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

fn edge_orientation(cube: &Cube) -> u32 {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn random_cubes(count: usize, mut f: impl FnMut(&Cube) -> ()) {
        use rand::seq::*;
        let mut rng = rand::thread_rng();

        let mut cube = Cube::solved();
        for _ in 0..count {
            let move_ = Move::all().choose(&mut rng).unwrap();
            cube = cube.apply(move_);
            f(&cube);
        }
    }

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

        #[test]
        fn all_domino_moves_are_zero() {
            use rand::seq::*;
            let mut rng = rand::thread_rng();

            let mut cube = Cube::solved();
            for _ in 0..100_000 {
                let move_ = domino_moves().choose(&mut rng).unwrap();
                cube = cube.apply(move_);
                assert_eq!(edge_orientation(&cube), 0, "{}\n{}", move_, cube);
            }
        }

        #[test]
        fn always_less_than_2_pow_11() {
            random_cubes(100_000, |cube| {
                let coord = edge_orientation(&cube);
                assert!(coord < 2u32.pow(11), "{}\n{}", coord, cube);
            });
        }
    }

    #[cfg(test)]
    mod heuristic_table {
        use super::*;

        fn simple_evaluator(moves: &[Move]) -> Duration {
            Duration::from_millis(10) * (moves.len() as u32)
        }

        lazy_static::lazy_static! {
            static ref CORNER_ORIENTATION: HeuristicTable<u32> = HeuristicTable::init(
                "corner_orientation",
                corner_orientation,
                &Move::all().collect::<Vec<_>>(),
                &simple_evaluator,
                None,
            );
        }

        #[test]
        fn has_quickcheck_generated() {
            let cube = Cube::solved().apply_all(Move::parse_sequence("R' F2 U'").unwrap());
            assert!(CORNER_ORIENTATION.has(&cube));
        }

        #[test]
        fn has_sune() {
            let cube =
                Cube::solved().apply_all(Move::parse_sequence("R U' R' U' R U2 R'").unwrap());
            assert!(CORNER_ORIENTATION.has(&cube));
        }

        #[quickcheck]
        fn is_exhaustive(moves: Vec<Move>) -> bool {
            let cube = Cube::solved().apply_all(moves);
            CORNER_ORIENTATION.has(&cube)
        }
    }
}
