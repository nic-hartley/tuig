use core::{fmt, ops::Range};

use tuig_iosys::{Action, XY};

/// Boundaries of something, i.e. a position and size.
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Bounds {
    pub pos: XY,
    pub size: XY,
}

impl Bounds {
    pub fn new(x: usize, y: usize, w: usize, h: usize) -> Self {
        Self {
            pos: XY(x, y),
            size: XY(w, h),
        }
    }

    /// Cut off the leftmost `amt` columns. Returns `(left, rest)`.
    pub fn split_left(&self, amt: usize) -> (Bounds, Bounds) {
        assert!(amt > 0 && amt <= self.size.x());
        let left = Bounds {
            pos: self.pos,
            size: XY(amt, self.size.y()),
        };
        let rest = Bounds {
            pos: self.pos + XY(amt, 0),
            size: self.size - XY(amt, 0),
        };
        (left, rest)
    }

    /// Cut off the rightmost `amt` columns. Returns `(right, rest)`. (**Not `(rest, right)`!**)
    pub fn split_right(&self, amt: usize) -> (Bounds, Bounds) {
        assert!(amt > 0 && amt <= self.size.x());
        let inverse = self.size.x() - amt;
        let (left, rest) = self.split_left(inverse);
        (rest, left)
    }

    /// Cut off the topmost `amt` columns. Returns `(top, rest)`.
    pub fn split_top(&self, amt: usize) -> (Bounds, Bounds) {
        assert!(amt > 0 && amt <= self.size.y());
        let top = Bounds {
            pos: self.pos,
            size: XY(self.size.x(), amt),
        };
        let rest = Bounds {
            pos: self.pos + XY(0, amt),
            size: self.size - XY(0, amt),
        };
        (top, rest)
    }

    /// Cut off the bottommost `amt` columns. Returns `(bottom, rest)`. (**Not `(rest, bottom)`!**)
    pub fn split_bottom(&self, amt: usize) -> (Bounds, Bounds) {
        assert!(amt > 0 && amt <= self.size.y());
        let inverse = self.size.y() - amt;
        let (top, rest) = self.split_top(inverse);
        (rest, top)
    }

    fn contains(&self, pos: XY) -> bool {
        self.xs().contains(&pos.x()) && self.ys().contains(&pos.y())
    }

    /// Filters out [`Action`]s which didn't occur in this `Bounds`.
    pub fn filter(&self, action: &Action) -> Action {
        match action.position() {
            Some(pos) if !self.contains(pos) => Action::Redraw,
            _ => action.clone(),
        }
    }

    /// The X coordinates within these bounds.
    pub fn xs(&self) -> Range<usize> {
        self.pos.x()..(self.pos.x() + self.size.x())
    }

    /// The Y coordinates within these bounds.
    pub fn ys(&self) -> Range<usize> {
        self.pos.y()..(self.pos.y() + self.size.y())
    }

    /// Boundaries containing nothing at all.
    pub fn empty() -> Self {
        Self {
            pos: XY(0, 0),
            size: XY(0, 0),
        }
    }
}

impl fmt::Debug for Bounds {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Bounds")
            .field("x", &self.pos.x())
            .field("y", &self.pos.y())
            .field("w", &self.size.x())
            .field("h", &self.size.y())
            .finish()
    }
}
