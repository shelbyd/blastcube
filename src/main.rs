mod challenge;
mod cube;
mod r#move;
mod solver;

mod prelude;
use prelude::*;

use std::time::Instant;

fn main() -> anyhow::Result<()> {
    let scrambles = [
        "R2 U' L2 R2 B2 F2 L2 F2 U2 L2",
        "R2 U' L' R2 B2 F' L F2 U2 L' U' B D U2 L2 D2 U R' B F' L R F U R2 B' F2 L2 U' L",
    ]
    .into_iter()
    .map(|s| Move::parse_sequence(s))
    .collect::<Result<Vec<_>, _>>()?;

    let scramble = &scrambles[0];
    eprintln!(
        "scramble: {}",
        scramble
            .iter()
            .map(Move::to_string)
            .collect::<Vec<_>>()
            .join(" ")
    );
    eprintln!("scramble.len(): {:#?}", scramble.len());

    let cube = Cube::solved().apply_all(scramble.iter().cloned());
    eprintln!("initial cube:\n{}", cube);

    let challenge = Challenge {
        inspection: Duration::default(),
        evaluator: |seq: &[_]| Duration::from_millis(100) * (seq.len() as u32),
    };

    let solver = solver::Kociemba::init(challenge);

    let started_at = Instant::now();
    let mut result_cube = cube.clone();

    eprintln!("Starting solve");
    for move_ in solver.solve(cube) {
        result_cube = result_cube.apply(move_);
        eprintln!("{:?} - {}", started_at.elapsed(), move_);
    }

    if result_cube == Cube::solved() {
        eprintln!("Solved in {:?}", started_at.elapsed());
    } else {
        eprintln!("DNF in {:?}", started_at.elapsed());
        eprintln!("final cube:\n{}", result_cube);
    }

    Ok(())
}
