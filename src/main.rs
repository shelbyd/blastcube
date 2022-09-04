mod challenge;
mod cube;
mod r#move;
mod solver;

mod prelude;
use prelude::*;

use std::time::Instant;

fn main() -> anyhow::Result<()> {
    // let scramble: Vec<Move> = Move::parse_sequence(
    //     "R2 U' L' R2 B2 F' L F2 U2 L' U' B D U2 L2 D2 U R' B F' L R F U R2 B' F2 L2 U' L",
    // )?;
    let scramble: Vec<Move> = Move::parse_sequence("R2 D F'")?;

    let cube = Cube::solved().apply_all(scramble);
    eprintln!("initial cube:\n{}", cube);

    let challenge = Challenge {
        inspection: Duration::default(),
        evaluator: |seq: &[Move]| Duration::from_millis(100) * (seq.len() as u32),
    };
    let solver = solver::NaiveIddfs::init(challenge);

    let started_at = Instant::now();
    let mut solved = cube.clone();
    for move_ in solver.solve(cube) {
        solved = solved.apply(move_);
        eprintln!("{:?}: {:?}", started_at.elapsed(), move_);
    }

    assert_eq!(solved, Cube::solved());

    Ok(())
}
