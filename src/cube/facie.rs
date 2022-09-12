use crate::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Location {
    Center(Face),
    Edge(Face, Face),
    Corner(Face, Face, Face),
}

impl Location {
    pub fn all() -> impl Iterator<Item = Location> {
        let centers = || all_faces();
        let edges = || {
            centers().flat_map(|a| {
                all_faces()
                    .filter(move |b| !Face::same_axis(a, *b) && a < *b)
                    .map(move |b| (a, b))
            })
        };
        let corners = || {
            edges().flat_map(|(a, b)| {
                all_faces()
                    .filter(move |c| b < *c)
                    .filter(move |c| !Face::same_axis(a, *c) && !Face::same_axis(b, *c))
                    .map(move |c| (a, b, c))
            })
        };

        centers()
            .map(Location::Center)
            .chain(edges().flat_map(|(a, b)| [Location::Edge(a, b), Location::Edge(b, a)]))
            .chain(corners().flat_map(|(a, b, c)| {
                [
                    Location::Corner(a, b, c),
                    Location::Corner(b, a, c),
                    Location::Corner(c, a, b),
                ]
            }))
    }
}

fn all_faces() -> impl Iterator<Item = Face> {
    enum_iterator::all()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_locations_in_cubie_order() {
        let mut locs = Location::all();

        while let Some(first) = locs.next() {
            match first {
                Location::Center(_) => {}

                Location::Edge(a, b) => {
                    assert_eq!(locs.next(), Some(Location::Edge(b, a)));
                }

                Location::Corner(a, b, c) => {
                    assert_eq!(locs.next(), Some(Location::Corner(b, a, c)));
                    assert_eq!(locs.next(), Some(Location::Corner(c, a, b)));
                }
            }
        }
    }

    #[test]
    fn all_locations_is_all() {
        assert_eq!(Location::all().count(), 9 * 6);
    }
}
