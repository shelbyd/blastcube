use crate::cube::coord::CoordCube;
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
        CoordCube::init_table();

        Kociemba {
            to_domino: {
                let moves = Move::all().collect::<Vec<_>>();
                let heuristics: Vec<Box<dyn Heuristic>> = vec![
                    Box::new(HeuristicTable::init(
                        "corner_orientation",
                        |c| c.corner_orientation(),
                        &moves,
                        &challenge.evaluator,
                        None,
                    )),
                    Box::new(HeuristicTable::init(
                        "edge_orientation",
                        |c| c.edge_orientation(),
                        &moves,
                        &challenge.evaluator,
                        None,
                    )),
                ];
                Phase::init(moves, is_domino_cube, heuristics)
            },
            post_domino: {
                let moves = domino_moves().collect::<Vec<_>>();
                let heuristics: Vec<Box<dyn Heuristic>> = vec![Box::new(HeuristicTable::init(
                    "corner_position",
                    |c| c.corner_position(),
                    &moves,
                    &challenge.evaluator,
                    Some(Duration::from_millis(3000)),
                ))];
                Phase::init(moves, |c| *c == Cube::solved(), heuristics)
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
        let cube = CoordCube::from(cube.clone().apply_all(prefix.clone()));

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
        cube: &CoordCube,
        move_stack: &mut Vec<Move>,
        phase: &Phase,
    ) -> Search {
        let min_time = phase.min_time(cube);
        let this_time = self.challenge.evaluator.eval(move_stack) + min_time;
        if this_time > max_time {
            return Search::NotFound(this_time);
        }

        if min_time == Duration::default() && phase.is_finished(&cube.raw) {
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

    fn min_time(&self, cube: &CoordCube) -> Duration {
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
    fn min_time(&self, cube: &CoordCube) -> Duration;
}

struct HeuristicTable<T: Eq + Hash, F> {
    name: String,
    exhaustive: bool,

    map: HashMap<T, Duration>,
    simplifier: F,
}

impl<T: Eq + Hash + core::fmt::Debug, F> HeuristicTable<T, F>
where
    F: Fn(&CoordCube) -> T,
{
    fn init(
        name: &str,
        simplifier: F,
        allowed_moves: &[Move],
        evaluator: &impl Evaluator,
        max_setup: Option<Duration>,
    ) -> Self {
        let mut result = Self {
            name: name.to_string(),
            exhaustive: true,

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
                Some(max) if start.elapsed() >= max => {
                    result.exhaustive = false;
                    true
                }
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

        let value = self.simplify_depr(&Cube::solved().apply_all(move_stack.clone()));
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
        let simplified = self.simplify_depr(cube);
        self.map.contains_key(&simplified)
    }

    fn simplify(&self, cube: &CoordCube) -> T {
        (self.simplifier)(cube)
    }

    fn simplify_depr(&self, cube: &Cube) -> T {
        (self.simplifier)(&CoordCube::from(cube.clone()))
    }
}

impl<T, F> Heuristic for HeuristicTable<T, F>
where
    T: Eq + Hash + Sync + Send + core::fmt::Debug,
    F: Fn(&CoordCube) -> T + Sync + Send,
{
    fn min_time(&self, cube: &CoordCube) -> Duration {
        let value = self.simplify(cube);
        if let Some(d) = self.map.get(&value) {
            return *d;
        }

        if self.exhaustive {
            panic!(
                "{}: missing value ({:?}) for cube\n{:?}",
                self.name, value, cube
            );
        }
        Duration::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    mod heuristic_table {
        use super::*;

        fn simple_evaluator(moves: &[Move]) -> Duration {
            Duration::from_millis(10) * (moves.len() as u32)
        }

        lazy_static::lazy_static! {
            static ref CORNER_ORIENTATION:
                    HeuristicTable<u16, Box<dyn Fn(&CoordCube) -> u16 + Sync + Send>>
            = HeuristicTable::init(
                "corner_orientation",
                Box::new(|c| c.corner_orientation()),
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
