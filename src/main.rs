mod challenge;
mod cube;
mod r#move;
mod solver;

mod prelude;
use prelude::*;

use std::time::Instant;

fn main() -> anyhow::Result<()> {
    simple_logger::SimpleLogger::new().init().unwrap();

    let scrambles = [
        "R2 U' L2 R2 B2 F2 L2 F2 U' L' B D F R2 F",
        "R2 U' L' R2 B2 F' L F2 U2 L' U' B D U2 L2 D2 U R' B F' L R F U R2 B' F2 L2 U' L",
    ]
    .into_iter()
    .map(|s| Move::parse_sequence(s))
    .collect::<Result<Vec<_>, _>>()?;

    let scramble = &scrambles[0];
    log::info!(
        "scramble: {}",
        scramble
            .iter()
            .map(Move::to_string)
            .collect::<Vec<_>>()
            .join(" ")
    );
    log::info!("scramble.len(): {:#?}", scramble.len());

    let cube = Cube::solved().apply_all(scramble.iter().cloned());
    log::info!("initial cube:\n{}", cube);

    let challenge = Challenge {
        inspection: Duration::default(),
        evaluator: |seq: &[_]| Duration::from_millis(100) * (seq.len() as u32),
    };

    let solver = std::sync::Arc::new(solver::Kociemba::init(challenge));

    let started_at = Instant::now();
    let mut result_cube = cube.clone();

    log::info!("Starting solve");
    for move_ in solver.solve(cube) {
        result_cube = result_cube.apply(move_);
        log::info!("{:?} - {}", started_at.elapsed(), move_);
    }

    if result_cube == Cube::solved() {
        log::info!("Solved in {:?}", started_at.elapsed());
    } else {
        log::info!("DNF in {:?}", started_at.elapsed());
        log::info!("final cube:\n{}", result_cube);
    }

    Ok(())
}
