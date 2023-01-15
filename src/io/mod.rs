//! Common code and types between input and output.

use std::{
    fmt,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Rem, RemAssign, Sub, SubAssign},
};

pub mod clifmt;
pub mod helpers;
pub mod input;
pub mod output;
pub mod sys;
pub mod widgets;

/// A position or size, with an X and a Y component.
///
/// You can do most arithmetic with `XY` that you could with integers, both elementwise with other `XY`s (e.g.
/// `XY(2, 3) * XY(4, 5) == XY(8, 15)`) and with scalars (e.g. `XY(2, 3) * 4 == XY(8, 12)`).
///
/// `XY`s aren't totally ordered because the components can be ordered differently, e.g. `XY(1, 5)` and `XY(2, 3)`,
/// the x is less but the y is greater. However, some methods (where it makes sense) are provided separately from
/// [`Ord`], and they operate elementwise, e.g. [`Self::clamp`].
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
    pub fn clamp(self, min: Self, max: Self) -> XY {
        XY(self.0.clamp(min.0, max.0), self.1.clamp(min.1, max.1))
    }
}

macro_rules! xy_op {
    ( $(
        $trait:ident($fn:ident) => $op:tt $assn_op:tt
    ),* $(,)? ) => {
        $(
            impl $trait for XY {
                type Output = XY;
                fn $fn(self, rhs: XY) -> XY {
                    XY(self.0 $op rhs.0, self.1 $op rhs.1)
                }
            }

            impl $trait<(usize, usize)> for XY {
                type Output = XY;
                fn $fn(self, rhs: (usize, usize)) -> XY {
                    XY(self.0 $op rhs.0, self.1 $op rhs.1)
                }
            }

            impl $trait<usize> for XY {
                type Output = XY;
                fn $fn(self, rhs: usize) -> XY {
                    XY(self.0 $op rhs, self.1 $op rhs)
                }
            }

            paste::paste! {
                impl [< $trait Assign >] for XY {
                    fn [< $fn _assign >] (&mut self, rhs: XY) {
                        self.0 $assn_op rhs.0;
                        self.1 $assn_op rhs.1;
                    }
                }
                impl [< $trait Assign >] <(usize, usize)> for XY {
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
    Add(add) => + +=,
    Sub(sub) => - -=,
    Mul(mul) => * *=,
    Div(div) => / /=,
    Rem(rem) => % %=,
}

impl fmt::Display for XY {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.0, self.1)
    }
}

impl fmt::Debug for XY {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "XY({}, {})", self.0, self.1)
    }
}

impl From<(usize, usize)> for XY {
    fn from(f: (usize, usize)) -> XY {
        XY(f.0, f.1)
    }
}

impl Into<(usize, usize)> for XY {
    fn into(self) -> (usize, usize) {
        (self.0, self.1)
    }
}

impl PartialOrd for XY {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let o0 = self.0.cmp(&other.0);
        let o1 = self.1.cmp(&other.1);
        if o0 == o1 {
            Some(o0)
        } else {
            None
        }
    }
}
