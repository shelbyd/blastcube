use crate::prelude::*;

use core::{cmp::Ordering, hash::Hash};
use std::collections::HashMap;

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
                let heuristics = vec![Box::new(HeuristicTable::init(
                    corner_twist_coord,
                    &moves,
                    &challenge.evaluator,
                )) as Box<dyn Heuristic>];
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

trait Heuristic {
    fn min_time(&self, cube: &Cube) -> Duration;
}

struct HeuristicTable<T: Eq + Hash> {
    map: HashMap<T, Duration>,
    simplifier: fn(&Cube) -> T,
}

impl<T: Eq + Hash> HeuristicTable<T> {
    fn init(
        simplifier: fn(&Cube) -> T,
        allowed_moves: &[Move],
        evaluator: &impl Evaluator,
    ) -> Self {
        let mut result = Self {
            simplifier,
            map: HashMap::default(),
        };

        for depth in 0..21 {
            if !result.expand_to_depth(depth, &mut Vec::new(), evaluator, allowed_moves) {
                log::info!(
                    "Finished expanding at depth {}, {} items",
                    depth,
                    result.map.len()
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
        let cube = (self.simplifier)(&Cube::solved().apply_all(Move::inverse_seq(move_stack)));
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
}

impl<T> Heuristic for HeuristicTable<T>
where
    T: Eq + Hash,
{
    fn min_time(&self, cube: &Cube) -> Duration {
        self.map
            .get(&(self.simplifier)(cube))
            .cloned()
            .unwrap_or_default()
    }
}

fn corner_twist_coord(cube: &Cube) -> usize {
    Location::all().fold(0, |v, loc| {
        let value = match loc {
            Location::Center(_) | Location::Edge(_, _) => return v,
            Location::Corner(_, _, _) if !matches!(cube.get(loc), Face::Up | Face::Down) => {
                return v
            }
            Location::Corner(Face::Up | Face::Down, _, _) => 0,
            Location::Corner(_, Face::Up | Face::Down, _) => 1,
            Location::Corner(_, _, Face::Up | Face::Down) => 2,
            Location::Corner(_, _, _) => unreachable!("{:?}", loc),
        };
        v * 3 + value
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    mod corner_twitst_coord {
        use super::*;

        #[test]
        fn solved_is_zero() {
            assert_eq!(corner_twist_coord(&Cube::solved()), 0);
        }

        #[test]
        fn right_turn_is_non_zero() {
            assert_ne!(
                corner_twist_coord(&Cube::solved().apply("R".parse().unwrap())),
                0
            );
        }

        #[test]
        fn left_is_not_right() {
            assert_ne!(
                corner_twist_coord(&Cube::solved().apply("R".parse().unwrap())),
                corner_twist_coord(&Cube::solved().apply("L".parse().unwrap())),
            );
        }

        #[test]
        fn double_right_is_zero() {
            assert_eq!(
                corner_twist_coord(&Cube::solved().apply("R2".parse().unwrap())),
                0
            );
        }

        #[test]
        fn same_cubies_opposite_twists() {
            let cw_twist = Move::parse_sequence("L D2 L' F' D2 F U F' D2 F L D2 L'").unwrap();
            let ccw_twist = Move::parse_sequence("F' D2 F L D2 L' U L D2 L' F' D2 F").unwrap();

            assert_ne!(
                corner_twist_coord(&Cube::solved().apply_all(cw_twist)),
                corner_twist_coord(&Cube::solved().apply_all(ccw_twist)),
            );
        }
    }
}
