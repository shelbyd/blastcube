mod cube;
mod r#move;

mod prelude;
use prelude::*;

fn main() -> anyhow::Result<()> {
    let scramble: Vec<Move> = Move::parse_sequence(
        "R2 U' L' R2 B2 F' L F2 U2 L' U' B D U2 L2 D2 U R' B F' L R F U R2 B' F2 L2 U' L",
    )?;

    let cube = Cube::solved().apply_all(scramble);
    dbg!(cube);

    Ok(())
}
