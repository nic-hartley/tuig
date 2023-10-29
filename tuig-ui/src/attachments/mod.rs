//! Builtin UI elements.
//!
//! Some of these also have dedicated convenience methods on [`Region`], which are generally preferred to using the
//! types directly. That said, all those convenience methods do is call `Region::attach` on an object in this module,
//! and if you need more control you might need to do the same.

mod button;
pub use button::Button;
mod textbox;
pub use textbox::{Textbox, TextboxData};
mod text_input;
pub use text_input::{TextInput, TextInputResult};
use tuig_iosys::Action;

use super::{Region, ScreenView};

/// Something that can be put into a [`Region`].
///
/// Deciding when to implement `Attachment` -- versus just fiddling with regions in the parent attachment -- can be
/// difficult, like most software design problems. It's a balancing act between the extra boilerplate and the extra
/// segmentation. Follow the single-responsibility principle, keep your code DRY, etc.
///
/// Implementing this trait directly will let you fiddle with `Region`s and, generally, is the high-level interface.
/// If you need the lower-level interface to actually handle input and draw to the character grid, you'll need to impl
/// [`RawAttachment`], or make use of `FnOnce(Action, ScreenView)` implementing `RawAttachment`.
pub trait Attachment<'s> {
    type Output;
    fn attach(self, region: Region<'s>) -> Self::Output;
}

/// The low-level "raw" trait for implementing [`Attachment`]s.
///
/// You should implement this at the lowest level of the hierarchy that it makes sense to. Remember you can pass the
/// same state into two elements -- which might let you reuse the region-splitting code, for example, or attach a
/// [`Textbox`] where you might otherwise have had to rewrite that logic yourself. For example, if you need an action
/// log and a map, you can probably implement the action log as a textbox and the map as a `RawAttachment`, rather
/// than both as one `RawAttachment` together.
///
/// Because `FnOnce(Action, ScreenView) -> T` implements `RawAttachment<Output=T>`, you can usually avoid implementing
/// this explicitly. For example, you might [`impl Attachment`][Attachment], writing a few lines of code in a lambda
/// rather than a full new type and `impl` block.
pub trait RawAttachment<'s> {
    type Output;
    fn raw_attach(self, input: Action, screen: ScreenView<'s>) -> Self::Output;
}

impl<'s, RAO, RA: RawAttachment<'s, Output = RAO>> Attachment<'s> for RA {
    type Output = RAO;
    fn attach(self, region: Region<'s>) -> Self::Output {
        let (input, screen) = region.raw_pieces();
        self.raw_attach(input, screen)
    }
}

impl<'s, T, F: FnOnce(Action, ScreenView<'s>) -> T> RawAttachment<'s> for F {
    type Output = T;
    fn raw_attach(self, input: Action, screen: ScreenView<'s>) -> Self::Output {
        self(input, screen)
    }
}

#[cfg(test)]
pub(crate) mod test_utils {
    use core::ops::{Bound, RangeBounds};

    use tuig_iosys::{
        fmt::{Cell, Color, Formatted, FormattedExt, Text},
        Screen,
    };

    const FILLER: &str = "0123456789abcdef";

    pub fn charat(x: usize, y: usize) -> char {
        FILLER.chars().nth((x * 5 + y * 3) % FILLER.len()).unwrap()
    }

    /// Generate a screen filled with miscellaneous "random" data, to fairly reliably check that stuff was left alone
    /// by the code under test, and offer a region of the given size and position within it.
    macro_rules! __make_screen {
        (
            $screen:ident($sx:literal, $sy:literal)
            $( , $region:ident($rx:tt, $ry:tt, $rw:tt, $rh:tt $( , $act:expr )?) )?
        ) => {
            #[allow(unused)]
            let mut $screen = Screen::new(XY($sx, $sy));
            for px in 0..$sx {
                for py in 0..$sy {
                    $screen[py][px] = Cell::of(charat(px, py)).on_black();
                }
            }
            $(
                make_region!($screen => $region($rx, $ry, $rw, $rh $( , $act )?))
            )?
        };
    }

    /// Generate a region within a screen, possibly with input.
    macro_rules! __make_region {
        (
            $screen:ident => $region:ident($rx:tt, $ry:tt, $rw:tt, $rh:tt $(, $act:expr )?)
        ) => {
            #[allow(unused)]
            let root = Region::new(&mut $screen, make_region!(@@select $( $act; )? Action::Redraw));
            let [_, vert] = root.split(crate::cols!($rx $rw))
                .expect("not enough space for desired cols");
            let [_, hori] = vert.split(crate::rows!($ry $rh))
                .expect("not enough space for desired rows");
            #[allow(unused_mut)]
            let mut $region = hori;
        };
        ( @@select $_1:expr $(; $_2:expr )? ) => { $_1 };
    }

    fn assert_cell_blank(s: &Screen, x: usize, y: usize) {
        let cell = &s[y][x];
        assert!(
            cell.ch == charat(x, y) && cell.get_fmt().bg == Color::Black,
            "mismatched cell at {}, {}: expected blank, got {:?}",
            x,
            y,
            cell,
        );
    }

    pub fn assert_area_blank(s: &Screen, x: impl RangeBounds<usize>, y: impl RangeBounds<usize>) {
        fn min(r: &impl RangeBounds<usize>) -> usize {
            match r.start_bound() {
                Bound::Included(v) => *v,
                Bound::Excluded(v) => v - 1,
                Bound::Unbounded => 0,
            }
        }
        fn max(r: &impl RangeBounds<usize>, m: usize) -> usize {
            match r.end_bound() {
                Bound::Included(v) => v + 1,
                Bound::Excluded(v) => *v,
                Bound::Unbounded => m,
            }
        }
        let min_x = min(&x);
        let max_x = max(&x, s.size().x());
        let min_y = min(&y);
        let max_y = max(&y, s.size().y());
        for x in min_x..max_x {
            for y in min_y..max_y {
                assert_cell_blank(s, x, y)
            }
        }
    }

    fn assert_cell_fmt(s: &Screen, x: usize, y: usize, c: Cell) {
        assert!(
            s[y][x] == c,
            "mismatched cell at {}, {}: expected {:?}, got {:?}",
            x,
            y,
            c,
            s[y][x]
        );
    }

    pub fn assert_area_fmt(s: &Screen, x: usize, y: usize, t: Text) {
        for (i, ch) in t.text.chars().enumerate() {
            assert_cell_fmt(s, x + i, y, Cell::of(ch).fmt_of(&t));
        }
    }

    macro_rules! __screen_assert {
        ( $sc:ident: $(
            $( fmt $fmt_x:expr, $fmt_y:expr, $text:literal $( $mod:ident )* )?
            $( blank $blank_x:expr, $blank_y:expr )?
        ),* ) => {
            $(
                $(
                    assert_area_fmt(&$sc, $fmt_x, $fmt_y, text1!($($mod)* $text));
                )?
                $(
                    assert_area_blank(&$sc, $blank_x, $blank_y);
                )?
            )*
        };
    }

    pub(crate) use {
        __make_region as make_region, __make_screen as make_screen,
        __screen_assert as screen_assert,
    };
}
