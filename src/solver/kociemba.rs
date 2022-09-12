use crate::prelude::*;

use core::{cmp::Ordering, hash::Hash};
use std::collections::{hash_map::HashMap, VecDeque};

pub struct Kociemba<E: Evaluator> {
    challenge: Challenge<E>,

    to_domino: Phase<Axis>,
    post_domino: Phase<Option<Face>>,
}

impl<E: Evaluator> Solver<E> for Kociemba<E> {
    fn init(challenge: Challenge<E>) -> Self {
        Kociemba {
            to_domino: {
                let moves = Move::all().collect::<Vec<_>>();
                Phase::init(moves, is_domino_cube, vec![])
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
        let to_domino = self.solve_to(&cube, &self.to_domino);
        log::info!("Domino path: {:?}", to_domino);
        let domino_cube = cube.apply_all(to_domino.clone());

        log::info!("Finding solution");
        let solution = self.solve_to(&domino_cube, &self.post_domino);
        Box::new(to_domino.into_iter().chain(solution.into_iter()))
    }
}

impl<E: Evaluator> Kociemba<E> {
    fn solve_to<F: Eq + Hash>(&self, cube: &Cube, phase: &Phase<F>) -> Vec<Move> {
        let mut best_time = Duration::default();
        loop {
            log::info!("Searching <= {:?}", best_time);
            match self.find_solution(best_time, &cube, &mut Vec::new(), phase) {
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
        let allowed_moves: Vec<_> = allowed_moves.into_iter().collect();

        Self {
            allowed_moves,
            finished_when,
            heuristics,
        }
    }

    fn min_time(&self, cube: &Cube) -> Duration {
        self.heuristics
            .iter()
            .map(|h| h.min_time(cube))
            .min()
            .unwrap_or_default()
    }

    fn is_finished(&self, cube: &Cube) -> bool {
        (self.finished_when)(cube)
    }
}

struct Heuristic<F: Eq + Hash> {
    map: HashMap<Cube<F>, Duration>,
    simplifier: fn(Cube) -> Cube<F>,
}

impl<F: Eq + Hash> Heuristic<F> {
    fn init(
        simplifier: fn(Cube) -> Cube<F>,
        allowed_moves: &[Move],
        evaluator: &impl Evaluator,
    ) -> Self {
        let allowed_moves: Vec<_> = allowed_moves.into_iter().collect();

        let mut map = HashMap::default();

        let mut explore = VecDeque::new();
        explore.push_back(Vec::new());
        while let Some(moves) = explore.pop_front() {
            if map.len() > 100_000 {
                break;
            }

            let cube = simplifier(Cube::solved().apply_all(Move::inverse_seq(&moves)));

            let min_time = evaluator.min_time(&moves);
            if let Some(t) = map.get(&cube) {
                if *t < min_time {
                    continue;
                }
            }
            map.insert(cube, min_time);

            for move_ in allowed_moves.iter() {
                let mut clone = moves.clone();
                clone.push(**move_);
                explore.push_back(clone);
            }
        }

        Self { simplifier, map }
    }

    fn min_time(&self, cube: &Cube) -> Duration {
        self.map
            .get(&(self.simplifier)(cube.clone()))
            .cloned()
            .unwrap_or_default()
    }
}

fn domino_axis(cube: Cube) -> Cube<Axis> {
    cube.map(|value| match value {
        Face::Up | Face::Down => Axis::Major,
        Face::Front | Face::Back => Axis::Minor,
        Face::Left | Face::Right => Axis::Ignored,
    })
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
enum Axis {
    Major,
    Minor,
    Ignored,
}

fn domino_face(cube: Cube) -> Cube<Option<Face>> {
    cube.map(|value| match value {
        Face::Up | Face::Down => Some(value),
        Face::Front | Face::Back => Some(value),
        Face::Left | Face::Right => None,
    })
}
