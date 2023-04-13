use core::cell::Cell;

use crate::{fmt, Action, Screen, XY};

use super::Bounds;

macro_rules! split_fn {
    ( $lt:lifetime: $( $name:ident ),* $(,)? ) => { $(
        pub fn $name(self, amt: usize) -> (Region<$lt>, Region<$lt>) {
            let Region { sd, input, bounds } = self;
            let (chunk, rest) = bounds.$name(amt);
            (
                Region { sd: sd.clone(), input: chunk.filter(&input), bounds: chunk },
                Region { sd: sd.clone(), input: rest.filter(&input), bounds: rest },
            )
        }
    )* }
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

    fn row(&self, pos: XY, width: usize) -> &[Cell<fmt::Cell>] {
        let start = self.index(pos);
        let end = start + width;
        &self.buffer[start..end]
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

    pub fn fill(&mut self, cell: fmt::Cell) {
        for y in self.bounds.ys() {
            for x in self.bounds.xs() {
                self.sd.cell(XY(x, y)).set(cell.clone());
            }
        }
    }

    split_fn!('s: split_left, split_right, split_top, split_bottom);
}
