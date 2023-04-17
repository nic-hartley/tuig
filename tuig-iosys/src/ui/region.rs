use core::cell::Cell;

use crate::{fmt, Action, Screen, XY};

use super::{Bounds, splitters::Splitter};

macro_rules! split_fn {
    ( $lt:lifetime: $( $name:ident ),* $(,)? ) => { paste::paste! { $(
        #[allow(dead_code)] // might as well have all of them, even if unused
        pub(crate) fn [<split_ $name>](self, amt: usize) -> (Region<$lt>, Region<$lt>) {
            let Region { sd, input, bounds } = self;
            let (chunk, rest) = bounds.[<split_ $name>](amt);
            (
                Region { sd: sd.clone(), input: chunk.filter(&input), bounds: chunk },
                Region { sd: sd.clone(), input: rest.filter(&input), bounds: rest },
            )
        }

        #[allow(dead_code)] // might as well have all of them, even if unused
        pub(crate) fn [<split_ $name _mut >](&mut self, amt: usize) -> Region<$lt> {
            let (chunk, rest) = self.bounds.[<split_ $name>](amt);
            let chunk_input = chunk.filter(&self.input);
            self.input = rest.filter(&self.input);
            self.bounds = rest;
            Region { sd: self.sd.clone(), input: chunk_input, bounds: chunk }
        }
    )* } }
}

/// Internal type used to bundle together the functionality around having mutable access to distinct subregions.
#[derive(Clone)]
struct ScreenData<'s> {
    buffer: &'s [Cell<fmt::Cell>],
    width: usize,
}

impl<'s> ScreenData<'s> {
    fn new(screen: &'s mut Screen) -> Self {
        let width = screen.size().x();
        let buffer = screen.cells.as_mut_slice();
        let buffer = Cell::from_mut(buffer).as_slice_of_cells();
        Self { buffer, width }
    }

    fn index(&self, pos: XY) -> usize {
        self.width * pos.y() + pos.x()
    }

    fn cell(&self, pos: XY) -> &Cell<fmt::Cell> {
        &self.buffer[self.index(pos)]
    }
}

pub struct Region<'s> {
    sd: ScreenData<'s>,
    pub input: Option<Action>,
    bounds: Bounds,
}

impl<'s> Region<'s> {
    pub fn new(screen: &'s mut Screen, input: Option<Action>) -> Self {
        let bounds = Bounds {
            pos: XY(0, 0),
            size: screen.size(),
        };
        Self {
            sd: ScreenData::new(screen),
            input,
            bounds,
        }
    }

    #[cfg(test)]
    pub(crate) fn bounds(&self) -> &Bounds {
        &self.bounds
    }

    pub fn size(&self) -> XY {
        self.bounds.size
    }

    pub fn set(&self, pos: XY, cell: fmt::Cell) {
        assert!(pos.x() < self.bounds.size.x(), "position out of bounds");
        assert!(pos.y() < self.bounds.size.y(), "position out of bounds");
        let realpos = self.bounds.pos + pos;
        self.sd.cell(realpos).set(cell);
    }

    pub fn fill(&self, cell: fmt::Cell) {
        for y in 0..self.bounds.size.y() {
            for x in 0..self.bounds.size.x() {
                self.set(XY(x, y), cell.clone());
            }
        }
    }

    split_fn!('s: left, right, top, bottom);

    pub fn split<S: Splitter<'s>>(self, splitter: S) -> S::Output {
        splitter.split(self)
    }
}
