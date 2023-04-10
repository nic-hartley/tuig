use crate::{XY, Action};

/// The boundaries of a [`Region`][super::Region].
pub struct Bounds {
    pub pos: XY,
    pub size: XY,
}

impl Bounds {
    /// Cut off the leftmost `amt` columns. Returns `(left, rest)`.
    pub fn split_left(self, amt: usize) -> (Bounds, Bounds) {
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

    /// Cut off the rightmost `amt` columns. Returns `(right, rest)`.
    pub fn split_right(self, amt: usize) -> (Bounds, Bounds) {
        let inverse = self.size.x() - amt;
        let (left, rest) = self.split_left(inverse);
        (rest, left)
    }

    /// Cut off the topmost `amt` columns. Returns `(top, rest)`.
    pub fn split_top(self, amt: usize) -> (Bounds, Bounds) {
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

    /// Cut off the bottommost `amt` columns. Returns `(bottom, rest)`.
    pub fn split_bottom(self, amt: usize) -> (Bounds, Bounds) {
        let inverse = self.size.y() - amt;
        let (top, rest) = self.split_top(inverse);
        (rest, top)
    }

    fn contains(&self, pos: XY) -> bool {
        let xs = self.pos.x()..(self.pos.x() + self.size.x());
        let ys = self.pos.y()..(self.pos.y() + self.size.y());
        xs.contains(&pos.x()) && ys.contains(&pos.y())
    }

    /// Filters out [`Action`]s which didn't occur in this `Bounds`.
    pub fn filter(&self, action: Option<Action>) -> Option<Action> {
        let action = action?;
        if let Some(pos) = action.position() {
            if !self.contains(pos) {
                // position event outside the Bounds, reject
                return None;
            }
        }
        // if there's no position, or the positin is in the Bounds, pass through
        Some(action)
    }
}
