use core::ops::Range;

use crate::{Action, XY};

/// The boundaries of a [`Region`][super::Region].
pub struct Bounds {
    pub pos: XY,
    pub size: XY,
}

impl Bounds {
    /// Cut off the leftmost `amt` columns. Returns `(left, rest)`.
    pub fn split_left(self, amt: usize) -> (Bounds, Bounds) {
        assert!(amt > 0 && amt < self.size.x());
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
        assert!(amt > 0 && amt < self.size.x());
        let inverse = self.size.x() - amt;
        let (left, rest) = self.split_left(inverse);
        (rest, left)
    }

    /// Cut off the topmost `amt` columns. Returns `(top, rest)`.
    pub fn split_top(self, amt: usize) -> (Bounds, Bounds) {
        assert!(amt > 0 && amt < self.size.y());
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
        assert!(amt > 0 && amt < self.size.y());
        let inverse = self.size.y() - amt;
        let (top, rest) = self.split_top(inverse);
        (rest, top)
    }

    fn contains(&self, pos: XY) -> bool {
        self.xs().contains(&pos.x()) && self.ys().contains(&pos.y())
    }

    /// Filters out [`Action`]s which didn't occur in this `Bounds`.
    pub fn filter(&self, action: &Option<Action>) -> Option<Action> {
        match action {
            Some(act) => match act.position() {
                Some(pos) if !self.contains(pos) => None,
                _other => Some(act.clone()),
            },
            None => None,
        }
    }

    pub fn xs(&self) -> Range<usize> {
        self.pos.x()..(self.pos.x() + self.size.x())
    }

    pub fn ys(&self) -> Range<usize> {
        self.pos.y()..(self.pos.y() + self.size.y())
    }
}

// #[cfg(test)]
// mod test {
//     // TODO: macro each_dir

//     #[test]
//     fn 
// }
