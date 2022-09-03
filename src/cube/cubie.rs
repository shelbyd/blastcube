use crate::prelude::*;

#[derive(Debug, PartialEq, Eq)]
pub struct CubieCube {

}

impl super::CubeLike for CubieCube {
    fn solved() -> Self {
        CubieCube {}
    }

    fn apply(self, move_: Move) -> Self {
        self
    }
}
