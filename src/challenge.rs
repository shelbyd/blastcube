use crate::prelude::*;

pub struct Challenge<E: Evaluator> {
    pub inspection: Duration,
    pub evaluator: E,
}

pub trait Evaluator {
    fn eval(&self, seq: &[Move]) -> Duration;
}

impl<F> Evaluator for F
where
    F: Fn(&[Move]) -> Duration,
{
    fn eval(&self, seq: &[Move]) -> Duration {
        (self)(seq)
    }
}
