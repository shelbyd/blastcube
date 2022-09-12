use crate::prelude::*;

pub struct Challenge<E: Evaluator> {
    pub inspection: Duration,
    pub evaluator: E,
}

// Other code assumes Evaluators are not super-linear.
//   E(a) + E(b) <= E(a + b)
pub trait Evaluator: Sync + Send + 'static {
    fn eval(&self, seq: &[Move]) -> Duration;

    fn min_time(&self, _seq: &[Move]) -> Duration {
        Duration::default()
    }
}

impl<F> Evaluator for F
where
    F: Send + Sync + 'static + Fn(&[Move]) -> Duration,
{
    fn eval(&self, seq: &[Move]) -> Duration {
        (self)(seq)
    }

    fn min_time(&self, seq: &[Move]) -> Duration {
        (self)(seq)
    }
}
