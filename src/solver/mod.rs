use crate::prelude::*;
use std::sync::Arc;

mod kociemba;
pub use kociemba::*;

mod naive_iddfs;
pub use naive_iddfs::*;

mod mitm;
pub use mitm::*;

pub trait Solver<E: Evaluator>: Sized {
    fn init(challenge: Challenge<E>) -> Self;

    fn solve(self: &Arc<Self>, cube: Cube) -> Box<dyn Iterator<Item = Move>>;
}
