use crate::prelude::*;

use core::cmp::Ordering;
use std::collections::{HashMap, VecDeque};

pub struct Kociemba<E: Evaluator> {
    challenge: Challenge<E>,

    to_domino: Phase<Axis>,
    post_domino: Phase<Option<Face>>,
}

impl<E: Evaluator> Solver<E> for Kociemba<E> {
    fn init(challenge: Challenge<E>) -> Self {
        Kociemba {
            challenge,

            to_domino: Phase::init(domino_axis, Move::all(), is_domino_cube),
            post_domino: Phase::init(domino_face, Move::all().filter(is_domino_move), |c| {
                *c == Cube::solved()
            }),
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
    fn is_domino(&self, cube: &Cube) -> bool {
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

    fn solve_to<F: PartialEq>(&self, cube: &Cube, phase: &Phase<F>) -> Vec<Move> {
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

    fn find_solution<F: PartialEq>(
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

struct Phase<F: 'static> {
    map: HashMap<Cube<F>, Duration>,
    allowed_moves: Vec<Move>,

    heuristic_filter: fn(Cube) -> Cube<F>,
    finished_when: fn(&Cube) -> bool,
}

impl<F> Phase<F> {
    fn init(
        heuristic_filter: fn(Cube) -> Cube<F>,
        allowed_moves: impl IntoIterator<Item = Move>,
        finished_when: fn(&Cube) -> bool,
    ) -> Self {
        let mut map = Default::default();

        let mut expand = VecDeque::new();
        expand.push_back(heuristic_filter(Cube::solved()));
        while let Some(cube) = expand.pop_front() {}

        Self {
            map,
            heuristic_filter,
            allowed_moves: allowed_moves.into_iter().collect(),
            finished_when,
        }
    }

    fn min_time(&self, cube: &Cube) -> Duration {
        Duration::default()
    }

    fn is_finished(&self, cube: &Cube) -> bool {
        (self.finished_when)(cube)
    }
}

fn domino_axis(cube: Cube) -> Cube<Axis> {
    cube.map(|value| match value {
        Face::Up | Face::Down => Axis::Major,
        Face::Front | Face::Back => Axis::Minor,
        Face::Left | Face::Right => Axis::Ignored,
    })
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
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
