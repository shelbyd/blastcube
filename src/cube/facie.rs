use crate::prelude::*;

#[derive(Debug, Clone, Copy)]
pub enum Location {
    Center(Face),
    Edge(Face, Face),
    Corner(Face, Face, Face),
}

impl Location {
    pub fn all() -> impl Iterator<Item = Location> {
        let centers = || all_faces();
        let edges = || {
            centers().flat_map(|major| {
                all_faces()
                    .filter(move |minor| !Face::same_axis(major, *minor))
                    .map(move |minor| (major, minor))
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
            .chain(edges().map(|(a, b)| Location::Edge(a, b)))
            .chain(corners().map(|(a, b, c)| Location::Corner(a, b, c)))
    }
}

fn all_faces() -> impl Iterator<Item = Face> {
    enum_iterator::all()
}
