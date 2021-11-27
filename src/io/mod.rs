//! Common code and types between input and output.

use std::{fmt, ops::{Add, AddAssign, Sub, SubAssign, Mul, MulAssign, Div, DivAssign}};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct XY(pub usize, pub usize);

impl XY {
    pub const fn x(&self) -> usize {
        self.0
    }

    pub const fn y(&self) -> usize {
        self.1
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

mod input;
pub use input::*;
pub use input::test as input_test;
mod output;
pub use output::*;
pub use output::test as output_test;
