use crate::prelude::*;

use std::collections::HashMap;

pub struct Mitm<E: Evaluator> {
    #[allow(unused)]
    challenge: Challenge<E>,
}

impl<E: Evaluator> super::Solver<E> for Mitm<E> {
    fn init(challenge: Challenge<E>) -> Self {
        Mitm { challenge }
    }

    fn solve(&self, cube: Cube) -> Box<dyn Iterator<Item = Move>> {
        let mut state = SolveState::default();
        for depth in 0..11usize {
            dbg!(depth);
            if let Some(solution) = state.expand(&cube) {
                dbg!(state.forward.len());
                dbg!(state.reverse.len());
                return Box::new(solution.into_iter());
            }
        }

        unreachable!();
    }
}

#[derive(Default)]
struct SolveState {
    forward: HashMap<Cube, Vec<Move>>,
    reverse: HashMap<Cube, Vec<Move>>,
}

impl SolveState {
    fn expand(&mut self, initial: &Cube) -> Option<Vec<Move>> {
        if self.forward.len() == 0 {
            assert_eq!(self.reverse.len(), 0);
            if *initial == Cube::solved() {
                return Some(Vec::new());
            }

            self.forward.insert(initial.clone(), Vec::new());
            self.reverse.insert(Cube::solved(), Vec::new());
            return None;
        }

        if let Some((forward, rev)) = Self::expand_mut(&mut self.forward, &mut self.reverse) {
            return Some(forward.into_iter().chain(reverse(rev)).collect());
        }

        if let Some((rev, forward)) = Self::expand_mut(&mut self.reverse, &mut self.forward) {
            return Some(forward.into_iter().chain(reverse(rev)).collect());
        }

        return None;
    }

    fn expand_mut(
        this: &mut HashMap<Cube, Vec<Move>>,
        other: &mut HashMap<Cube, Vec<Move>>,
    ) -> Option<(Vec<Move>, Vec<Move>)> {
        let expand = this
            .drain()
            .flat_map(|(cube, moves)| {
                Move::all().map(move |move_| {
                    (cube.clone().apply(move_), {
                        let mut m = moves.clone();
                        m.push(move_);
                        m
                    })
                })
            })
            .collect::<Vec<_>>();

        for (cube, moves) in expand {
            if let Some(other) = other.remove(&cube) {
                return Some((moves, other));
            }

            this.entry(cube).or_insert(moves);
        }

        return None;
    }
}

fn reverse(moves: Vec<Move>) -> Vec<Move> {
    if moves.len() == 0 {
        return moves;
    }

    moves.into_iter().rev().map(|m| m.reverse()).collect()
}
