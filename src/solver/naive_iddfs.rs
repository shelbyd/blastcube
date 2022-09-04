use crate::prelude::*;

pub struct NaiveIddfs<E: Evaluator> {
    challenge: Challenge<E>,
}

impl<E: Evaluator> NaiveIddfs<E> {
    fn find_solution(&self, remaining_moves: u8, cube: &Cube) -> Option<Vec<Move>> {
        if remaining_moves == 0 {
            if *cube == Cube::solved() {
                return Some(Vec::new());
            } else {
                return None;
            }
        }

        Move::all()
            .filter_map(|move_| {
                let c = cube.clone().apply(move_);
                self.find_solution(remaining_moves - 1, &c).map(|mut v| {
                    v.insert(0, move_);
                    v
                })
            })
            .min_by_key(|seq| self.challenge.evaluator.eval(seq))
    }
}

impl<E: Evaluator> super::Solver<E> for NaiveIddfs<E> {
    fn init(challenge: Challenge<E>) -> NaiveIddfs<E> {
        NaiveIddfs { challenge }
    }

    fn solve(&self, cube: Cube) -> Box<dyn Iterator<Item = Move>> {
        let mut move_depth = 1;

        loop {
            if let Some(solution) = self.find_solution(move_depth, &cube) {
                return Box::new(solution.into_iter());
            }
            move_depth += 1;
        }
    }
}
