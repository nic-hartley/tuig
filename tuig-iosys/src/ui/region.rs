use crate::{fmt::Cell, Action, Screen, XY};

use super::Bounds;

macro_rules! split_fn {
    ( $lt:lifetime: $( $name:ident ),* $(,)? ) => { $(
        pub fn $name(self, amt: usize) -> (Region<$lt>, Region<$lt>) {
            let Region { screen, input, bounds } = self;
            let (chunk, rest) = bounds.$name(amt);
            // SAFETY: The bounds are forced to be mutually exclusive by the `Bounds::split_*` methods called, so the
            // regions refer to different areas of the screen. The input region is consumed, so it can't produce
            // regions multiple times. Regions limit their effect to just their (guaranteed non-overlapping) bounds,
            // so keeping several mutable references to the screen is safe because
            let s = unsafe { &mut *(screen as *mut _) };
            (
                Region { screen, input: chunk.filter(&input), bounds: chunk },
                Region { screen: s, input: rest.filter(&input), bounds: rest },
            )
        }
    )* }
}

pub struct Region<'s> {
    screen: &'s mut Screen,
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
            screen,
            input,
            bounds,
        }
    }

    pub fn fill(&mut self, cell: Cell) {
        for y in self.bounds.ys() {
            for x in self.bounds.xs() {
                self.screen[y][x] = cell.clone();
            }
        }
    }

    split_fn!('s: split_left, split_right, split_top, split_bottom);
}
