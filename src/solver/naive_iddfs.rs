use crate::prelude::*;

use std::collections::VecDeque;

pub struct NaiveIddfs<E: Evaluator> {
    challenge: Challenge<E>,
}

impl<E: Evaluator> NaiveIddfs<E> {
    fn find_solution(
        &self,
        remaining_moves: u8,
        cube: &Cube,
        last_move: Option<Move>,
    ) -> Option<VecDeque<Move>> {
        if remaining_moves == 0 {
            if *cube == Cube::solved() {
                return Some(Default::default());
            } else {
                return None;
            }
        }

        Move::all()
            .filter(|move_| match last_move {
                None => true,
                Some(l) => l.face != move_.face,
            })
            .filter_map(|move_| {
                let c = cube.clone().apply(move_);
                let mut solution = self.find_solution(remaining_moves - 1, &c, Some(move_))?;
                solution.push_front(move_);
                Some(solution)
            })
            .min_by_key(|seq| {
                self.challenge
                    .evaluator
                    .eval(seq.iter().cloned().collect::<Vec<_>>().as_slice())
            })
    }
}

impl<E: Evaluator> super::Solver<E> for NaiveIddfs<E> {
    fn init(challenge: Challenge<E>) -> NaiveIddfs<E> {
        NaiveIddfs { challenge }
    }

    fn solve(self: &std::sync::Arc<Self>, cube: Cube) -> Box<dyn Iterator<Item = Move>> {
        (0..)
            .filter_map(|move_depth| self.find_solution(move_depth, &cube, None))
            .map(|seq| Box::new(seq.into_iter()))
            .next()
            .expect("every cube is eventually solvable")
    }
}
