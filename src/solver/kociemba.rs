use crate::prelude::*;

use core::cmp::Ordering;

pub struct Kociemba<E: Evaluator> {
    #[allow(dead_code)]
    challenge: Challenge<E>,
}

impl<E: Evaluator> Solver<E> for Kociemba<E> {
    fn init(challenge: Challenge<E>) -> Self {
        Kociemba { challenge }
    }

    fn solve(&self, cube: Cube) -> Box<dyn Iterator<Item = Move>> {
        log::info!("Finding domino");
        let to_domino = self.solve_to(
            &cube,
            |c| self.is_domino(c),
            &Move::all().collect::<Vec<_>>(),
        );
        log::info!("Domino path: {:?}", to_domino);
        let domino_cube = cube.apply_all(to_domino.clone());

        log::info!("Finding solution");
        let solution = self.solve_to(
            &domino_cube,
            |c| c == &Cube::solved(),
            &Move::all().filter(is_domino_move).collect::<Vec<_>>(),
        );
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

    fn solve_to(
        &self,
        cube: &Cube,
        pred: impl Fn(&Cube) -> bool,
        allowed_moves: &[Move],
    ) -> Vec<Move> {
        let mut best_time = Duration::default();
        loop {
            log::info!("Searching <= {:?}", best_time);
            match self.find_solution(best_time, &cube, &mut Vec::new(), &pred, allowed_moves) {
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
        pred: &impl Fn(&Cube) -> bool,
        allowed_moves: &[Move],
    ) -> Search {
        let this_time = self.challenge.evaluator.eval(move_stack);
        if this_time > max_time {
            return Search::NotFound(this_time);
        }

        if pred(cube) {
            return Search::Found(move_stack.clone());
        }

        let last_move = move_stack.last().cloned();
        allowed_moves
            .iter()
            .filter(|m| match last_move {
                None => true,
                Some(before) => m.could_follow(&before),
            })
            .fold(Search::NotFound(Duration::MAX), |best, &move_| {
                move_stack.push(move_);
                let cube = cube.clone().apply(move_);
                let sub = self.find_solution(max_time, &cube, move_stack, pred, allowed_moves);
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
