use crate::{Action, Screen, XY};

use super::{splitters::Splitter, Bounds, ScreenView};

macro_rules! split_fn {
    ( $lt:lifetime: $( $name:ident ),* $(,)? ) => { paste::paste! { $(
        #[allow(dead_code)] // might as well have all of them, even if unused
        pub(crate) fn [<split_ $name>](mut self, amt: usize) -> (Region<$lt>, Region<$lt>) {
            let chunk = self.[<split_ $name _mut>](amt);
            (chunk, self)
        }

        #[allow(dead_code)] // might as well have all of them, even if unused
        pub(crate) fn [<split_ $name _mut >](&mut self, amt: usize) -> Region<$lt> {
            let (chunk, rest) = self.bounds.[<split_ $name>](amt);
            let chunk_input = chunk.filter(&self.input);
            // SAFETY: `chunk` and `rest` are guaranteed to be non-overlapping by the `bounds.split_*` methods
            let [chunk_sv, rest_sv] = unsafe {
                core::mem::take(&mut self.sv).split([chunk, rest])
            };
            self.sv = rest_sv;
            self.input = rest.filter(&self.input);
            self.bounds = rest;
            Region { sv: chunk_sv, input: chunk_input, bounds: chunk }
        }
    )* } }
}

pub struct Region<'s> {
    sv: ScreenView<'s>,
    input: Action,
    bounds: Bounds,
}

impl<'s> Region<'s> {
    pub fn new(screen: &'s mut Screen, input: Action) -> Self {
        let bounds = Bounds::new(0, 0, screen.size().x(), screen.size().y());
        Self {
            sv: unsafe { ScreenView::new(screen, bounds) },
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

    split_fn!('s: left, right, top, bottom);

    pub fn split<S: Splitter<'s>>(self, splitter: S) -> S::Output {
        splitter.split(self)
    }

    pub(crate) fn raw_pieces(self) -> (Action, ScreenView<'s>) {
        (self.input, self.sv)
    }
}

impl Region<'static> {
    pub fn empty() -> Self {
        Self { sv: ScreenView::empty(), input: Action::Redraw, bounds: Bounds::new(0, 0, 0, 0) }
    }
}

impl<'s> core::fmt::Debug for Region<'s> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.bounds.fmt(f)
    }
}
