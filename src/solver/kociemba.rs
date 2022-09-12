use crate::prelude::*;

use core::{cmp::Ordering, hash::Hash};
use std::collections::HashMap;

pub struct Kociemba<E: Evaluator> {
    challenge: Challenge<E>,

    to_domino: Phase<usize>,
    post_domino: Phase<Option<Face>>,
}

impl<E: Evaluator> Solver<E> for Kociemba<E> {
    fn init(challenge: Challenge<E>) -> Self {
        Kociemba {
            to_domino: {
                let moves = Move::all().collect::<Vec<_>>();
                let heuristics = vec![Heuristic::init(
                    corner_twist_coord,
                    &moves,
                    &challenge.evaluator,
                )];
                Phase::init(moves, is_domino_cube, heuristics)
            },
            post_domino: {
                let moves = Move::all().filter(is_domino_move).collect::<Vec<_>>();
                Phase::init(moves, |c| *c == Cube::solved(), vec![])
            },

            challenge,
        }
    }

    fn solve(&self, cube: Cube) -> Box<dyn Iterator<Item = Move>> {
        log::info!("Finding domino");
        let to_domino = self.solve_to(&cube, &self.to_domino, Vec::new());
        log::info!("Domino path: {:?}", to_domino);

        log::info!("Finding solution");
        let solution = self.solve_to(&cube, &self.post_domino, to_domino);
        Box::new(solution.into_iter())
    }
}

impl<E: Evaluator> Kociemba<E> {
    fn solve_to<F: Eq + Hash>(
        &self,
        cube: &Cube,
        phase: &Phase<F>,
        mut prefix: Vec<Move>,
    ) -> Vec<Move> {
        let cube = cube.clone().apply_all(prefix.clone());

        let mut best_time = Duration::default();
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

    fn find_solution<F: Eq + Hash>(
        &self,
        max_time: Duration,
        cube: &Cube,
        move_stack: &mut Vec<Move>,
        phase: &Phase<F>,
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
            .filter(|m| match last_move {
                None => true,
                Some(before) => m.could_follow(&before),
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
    }
}

enum Search {
    NotFound(Duration),
    Found(Vec<Move>),
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

struct Phase<F: Eq + Hash> {
    allowed_moves: Vec<Move>,
    finished_when: fn(&Cube) -> bool,
    heuristics: Vec<Heuristic<F>>,
}

impl<F: Eq + Hash> Phase<F> {
    fn init(
        allowed_moves: impl IntoIterator<Item = Move>,
        finished_when: fn(&Cube) -> bool,
        heuristics: Vec<Heuristic<F>>,
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

struct Heuristic<T: Eq + Hash> {
    map: HashMap<T, Duration>,
    simplifier: fn(Cube) -> T,
}

impl<T: Eq + Hash> Heuristic<T> {
    fn init(simplifier: fn(Cube) -> T, allowed_moves: &[Move], evaluator: &impl Evaluator) -> Self {
        let mut result = Self {
            simplifier,
            map: HashMap::default(),
        };

        for depth in 0..21 {
            dbg!(depth, result.map.len());
            if !result.expand_to_depth(depth, &mut Vec::new(), evaluator, allowed_moves) {
                break;
            }
        }

        dbg!(result.map.len());
        result
    }

    fn expand_to_depth(
        &mut self,
        depth: usize,
        move_stack: &mut Vec<Move>,
        evaluator: &impl Evaluator,
        allowed_moves: &[Move],
    ) -> bool {
        let cube = (self.simplifier)(Cube::solved().apply_all(Move::inverse_seq(move_stack)));
        let time = evaluator.min_time(move_stack);

        if depth == 0 {
            if let Some(t) = self.map.get(&cube) {
                if *t <= time {
                    return false;
                }
            }

            self.map.insert(cube, time);
            return true;
        }

        if let Some(t) = self.map.get(&cube) {
            if *t < time {
                return false;
            }
        }

        allowed_moves.iter().fold(false, |any, move_| {
            move_stack.push(*move_);
            let result = self.expand_to_depth(depth - 1, move_stack, evaluator, allowed_moves);
            move_stack.pop();
            any || result
        })
    }

    fn min_time(&self, cube: &Cube) -> Duration {
        // Duration::default()
        self.map
            .get(&(self.simplifier)(cube.clone()))
            .cloned()
            .unwrap_or_default()
    }
}

fn corner_twist_coord(cube: Cube) -> usize {
    0
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
enum Axis {
    Major,
    Minor,
    Ignored,
}
