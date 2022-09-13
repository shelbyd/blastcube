use crate::prelude::*;

#[derive(Clone, Copy)]
pub struct BlastMachineEvaluator;

impl Evaluator for BlastMachineEvaluator {
    fn eval(&self, seq: &[Move]) -> Duration {
        let single_move_time = Duration::from_millis(10);
        let double_move_time = Duration::from_millis(14);

        let mut last_move: Option<Move> = None;
        seq.into_iter()
            .map(|move_| match (last_move.replace(*move_), move_) {
                (Some(last), m) if Face::same_axis(last.face, m.face) => Duration::default(),

                (
                    _,
                    Move {
                        direction: Direction::Double,
                        ..
                    },
                ) => double_move_time,
                (_, _) => single_move_time,
            })
            .sum()
    }

    fn min_time(&self, seq: &[Move]) -> Duration {
        match seq {
            [] => Duration::default(),
            [_] => Duration::default(),
            [_, internal @ .., _] => self.eval(internal),
        }
    }
}
