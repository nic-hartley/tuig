//! Just the `XY` type.

use core::{
    fmt,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Rem, RemAssign, Sub, SubAssign},
};

/// A position or size, with an X and a Y component.
///
/// You can do most arithmetic with `XY` that you could with integers, both elementwise with other `XY`s (e.g.
/// `XY(2, 3) * XY(4, 5) == XY(8, 15)`) and with scalars (e.g. `XY(2, 3) * 4 == XY(8, 12)`).
///
/// `XY`s aren't totally ordered because the components can be ordered differently, e.g. `XY(1, 5)` and `XY(2, 3)`,
/// the x is less but the y is greater. However, some methods (where it makes sense) are provided separately from
/// [`Ord`], and they operate elementwise, e.g. [`Self::clamp`].
///
/// When used as a position, `XY(0, 0)` is at the top left of the screen, and `XY(0, 1)` is just below it -- the usual
/// "graphics axes".
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct XY(pub usize, pub usize);

impl XY {
    /// The X component
    pub const fn x(&self) -> usize {
        self.0
    }

    /// The Y component
    pub const fn y(&self) -> usize {
        self.1
    }

    /// Contain this XY within the given bounds, elementwise.
    pub fn clamp(self, top_left: Self, bottom_right: Self) -> XY {
        let x = self.x().clamp(top_left.x(), bottom_right.x());
        let y = self.y().clamp(top_left.y(), bottom_right.y());
        XY(x, y)
    }
}

macro_rules! xy_op {
    ( $(
        $trait:ident($fn:ident) => $op:tt $assn_op:tt
        $( , $name:ident: $lhs:expr, $rhs:expr => $res:expr )*
    );* $(;)? ) => {
        $(
            impl $trait for XY {
                type Output = XY;
                #[cfg_attr(coverage, no_coverage)]
                fn $fn(self, rhs: XY) -> XY {
                    XY(self.0 $op rhs.0, self.1 $op rhs.1)
                }
            }

            impl $trait<(usize, usize)> for XY {
                type Output = XY;
                #[cfg_attr(coverage, no_coverage)]
                fn $fn(self, rhs: (usize, usize)) -> XY {
                    XY(self.0 $op rhs.0, self.1 $op rhs.1)
                }
            }

            impl $trait<usize> for XY {
                type Output = XY;
                #[cfg_attr(coverage, no_coverage)]
                fn $fn(self, rhs: usize) -> XY {
                    XY(self.0 $op rhs, self.1 $op rhs)
                }
            }

            paste::paste! {
                impl [< $trait Assign >] for XY {
                    #[cfg_attr(coverage, no_coverage)]
                    fn [< $fn _assign >] (&mut self, rhs: XY) {
                        self.0 $assn_op rhs.0;
                        self.1 $assn_op rhs.1;
                    }
                }
                impl [< $trait Assign >] <(usize, usize)> for XY {
                    #[cfg_attr(coverage, no_coverage)]
                    fn [< $fn _assign >] (&mut self, rhs: (usize, usize)) {
                        self.0 $assn_op rhs.0;
                        self.1 $assn_op rhs.1;
                    }
                }
            }
        )*
    };
}

xy_op! {
    Add(add) => + +=;
    Sub(sub) => - -=;
    Mul(mul) => * *=;
    Div(div) => / /=;
    Rem(rem) => % %=;
}

impl fmt::Display for XY {
    #[cfg_attr(coverage, no_coverage)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.0, self.1)
    }
}

impl fmt::Debug for XY {
    #[cfg_attr(coverage, no_coverage)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "XY({}, {})", self.0, self.1)
    }
}

impl From<(usize, usize)> for XY {
    #[cfg_attr(coverage, no_coverage)]
    fn from(f: (usize, usize)) -> XY {
        XY(f.0, f.1)
    }
}

impl From<XY> for (usize, usize) {
    #[cfg_attr(coverage, no_coverage)]
    fn from(val: XY) -> Self {
        (val.0, val.1)
    }
}

impl PartialOrd for XY {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        let o0 = self.0.cmp(&other.0);
        let o1 = self.1.cmp(&other.1);
        if o0 == o1 {
            Some(o0)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn xy_clamps_elementwise() {
        let tl = XY(2, 3);
        let br = XY(8, 7);
        assert_eq!(XY(1, 1).clamp(tl, br), XY(2, 3));
        assert_eq!(XY(4, 1).clamp(tl, br), XY(4, 3));
        assert_eq!(XY(9, 1).clamp(tl, br), XY(8, 3));
        assert_eq!(XY(1, 5).clamp(tl, br), XY(2, 5));
        assert_eq!(XY(4, 5).clamp(tl, br), XY(4, 5));
        assert_eq!(XY(9, 5).clamp(tl, br), XY(8, 5));
        assert_eq!(XY(1, 8).clamp(tl, br), XY(2, 7));
        assert_eq!(XY(4, 8).clamp(tl, br), XY(4, 7));
        assert_eq!(XY(9, 8).clamp(tl, br), XY(8, 7));
    }
}
