use crate::prelude::*;

mod naive_iddfs;
pub use naive_iddfs::*;

pub trait Solver<E: Evaluator>: Sized {
    fn init(challenge: Challenge<E>) -> Self;

    fn solve(&self, cube: Cube) -> Box<dyn Iterator<Item = Move>>;
}
