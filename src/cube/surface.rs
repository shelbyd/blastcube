use crate::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cube {
    up: Surface,
    down: Surface,
    front: Surface,
    back: Surface,
    left: Surface,
    right: Surface,
}

impl super::CubeLike for Cube {
    fn solved() -> Cube {
        Cube {
            up: Surface::from(Face::Up),
            down: Surface::from(Face::Down),
            front: Surface::from(Face::Front),
            back: Surface::from(Face::Back),
            left: Surface::from(Face::Left),
            right: Surface::from(Face::Right),
        }
    }

    fn apply(mut self, move_: Move) -> Self {
        self.rotate(move_);
        self
    }
}

impl Cube {
    pub fn rotate(&mut self, move_: impl Into<Move>) {
        let move_ = move_.into();

        let surface = self.surface_mut(move_.face);
        match move_.direction {
            Direction::Single => surface.rotate(),
            Direction::Reverse => surface.rotate_reverse(),
            Direction::Double => surface.rotate_double(),
        }

        let mut slices = self.slices(move_.face);

        match move_.direction {
            Direction::Single => {
                let first = slices[0].owned();
                slices[0].set(slices[1].owned());
                slices[1].set(slices[2].owned());
                slices[2].set(slices[3].owned());
                slices[3].set(first);
            }
            Direction::Reverse => {
                let last = slices[3].owned();
                slices[3].set(slices[2].owned());
                slices[2].set(slices[1].owned());
                slices[1].set(slices[0].owned());
                slices[0].set(last);
            }
            Direction::Double => {
                let temp = slices[0].owned();
                slices[0].set(slices[2].owned());
                slices[2].set(temp);

                let temp = slices[1].owned();
                slices[1].set(slices[3].owned());
                slices[3].set(temp);
            }
        }
    }

    fn surface_mut(&mut self, face: Face) -> &mut Surface {
        match face {
            Face::Up => &mut self.up,
            Face::Down => &mut self.down,
            Face::Left => &mut self.left,
            Face::Right => &mut self.right,
            Face::Front => &mut self.front,
            Face::Back => &mut self.back,
        }
    }

    #[inline(never)]
    fn slices(&mut self, face: Face) -> [SliceMut; 4] {
        match face {
            Face::Up => [
                self.left.top_mut(),
                self.back.top_mut(),
                self.right.top_mut(),
                self.front.top_mut(),
            ],
            Face::Down => [
                self.left.bottom_mut(),
                self.front.bottom_mut(),
                self.right.bottom_mut(),
                self.back.bottom_mut(),
            ],
            Face::Front => [
                self.up.bottom_mut(),
                self.right.left_mut(),
                self.down.top_mut(),
                self.left.right_mut(),
            ],
            Face::Back => [
                self.up.top_mut(),
                self.left.left_mut(),
                self.down.bottom_mut(),
                self.right.right_mut(),
            ],
            Face::Right => [
                self.up.right_mut(),
                self.back.left_mut(),
                self.down.right_mut(),
                self.front.right_mut(),
            ],
            Face::Left => [
                self.up.left_mut(),
                self.front.left_mut(),
                self.down.left_mut(),
                self.back.right_mut(),
            ],
        }
    }
}

impl std::fmt::Display for Cube {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let slices = |surface: &Surface, face: Face| {
            [
                surface.top(),
                surface.mid_horizontal(face),
                surface.bottom(),
            ]
        };

        for slice in slices(&self.up, Face::Up).iter() {
            writeln!(f, "    {}", slice)?;
        }

        let middle_slices = [
            (&self.left, Face::Left),
            (&self.front, Face::Front),
            (&self.right, Face::Right),
            (&self.back, Face::Back),
        ]
        .iter()
        .map(|(surface, face)| slices(surface, *face))
        .collect::<Vec<_>>();

        for index in 0..3 {
            for slice_list in &middle_slices {
                write!(f, "{} ", slice_list[index])?;
            }
            write!(f, "\n")?;
        }

        for slice in slices(&self.down, Face::Down).iter() {
            writeln!(f, "    {}", slice)?;
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
struct Surface([Face; 8]);

impl Surface {
    #[inline(never)]
    fn rotate(&mut self) {
        unsafe {
            let as_int: u64 = std::mem::transmute(*self);
            let rotated = as_int.rotate_left(16);
            *self = std::mem::transmute(rotated);
        }
    }

    #[inline(never)]
    fn rotate_reverse(&mut self) {
        unsafe {
            let as_int: u64 = std::mem::transmute(*self);
            let rotated = as_int.rotate_right(16);
            *self = std::mem::transmute(rotated);
        }
    }

    #[inline(never)]
    fn rotate_double(&mut self) {
        unsafe {
            let as_int: u64 = std::mem::transmute(*self);
            let rotated = as_int.rotate_right(32);
            *self = std::mem::transmute(rotated);
        }
    }

    fn top(&self) -> Slice {
        Slice([self.0[0], self.0[1], self.0[2]])
    }

    fn top_mut(&mut self) -> SliceMut {
        self.slice_mut(0, 1, 2)
    }

    #[inline(always)]
    fn slice_mut<'s>(&'s mut self, first: u8, second: u8, third: u8) -> SliceMut<'s> {
        assert!((first as usize) < self.0.len());
        assert!((second as usize) < self.0.len());
        assert!((third as usize) < self.0.len());
        assert!(first != second);
        assert!(first != third);
        assert!(second != third);

        SliceMut {
            surface: self,
            indices: [first, second, third],
        }
    }

    fn mid_horizontal(&self, face: Face) -> Slice {
        Slice([self.0[7], face, self.0[3]])
    }

    fn bottom(&self) -> Slice {
        Slice([self.0[6], self.0[5], self.0[4]])
    }

    fn bottom_mut(&mut self) -> SliceMut {
        self.slice_mut(4, 5, 6)
    }

    fn right_mut(&mut self) -> SliceMut {
        self.slice_mut(2, 3, 4)
    }

    fn left_mut(&mut self) -> SliceMut {
        self.slice_mut(6, 7, 0)
    }
}

impl From<Face> for Surface {
    fn from(face: Face) -> Surface {
        Surface([face; 8])
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
struct Slice([Face; 3]);

impl std::fmt::Display for Slice {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}{}{}", self.0[0], self.0[1], self.0[2])
    }
}

struct SliceMut<'s> {
    surface: &'s mut Surface,
    indices: [u8; 3],
}

impl<'s> SliceMut<'s> {
    fn owned(&self) -> Slice {
        use core::mem::{transmute, MaybeUninit};

        let mut array: [MaybeUninit<Face>; 3] = unsafe { MaybeUninit::uninit().assume_init() };
        for i in 0..3 {
            array[i] = MaybeUninit::new(self.surface.0[self.indices[i] as usize]);
        }
        Slice(unsafe { transmute::<_, [Face; 3]>(array) })
    }

    fn set(&mut self, owned: Slice) {
        for i in 0..3 {
            self.surface.0[self.indices[i] as usize] = owned.0[i];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotate_surface() {
        use Face::*;
        let mut surface = Surface([Left, Left, Up, Up, Right, Right, Down, Down]);
        surface.rotate();
        assert_eq!(
            surface,
            Surface([Down, Down, Left, Left, Up, Up, Right, Right])
        );
    }

    #[test]
    fn rotate_surface_reverse() {
        use Face::*;
        let mut surface = Surface([Left, Left, Up, Up, Right, Right, Down, Down]);
        surface.rotate_reverse();
        assert_eq!(
            surface,
            Surface([Up, Up, Right, Right, Down, Down, Left, Left])
        );
    }

    #[test]
    fn rotate_surface_double() {
        use Face::*;
        let mut surface = Surface([Left, Left, Up, Up, Right, Right, Down, Down]);
        surface.rotate_double();
        assert_eq!(
            surface,
            Surface([Right, Right, Down, Down, Left, Left, Up, Up])
        );
    }
}
