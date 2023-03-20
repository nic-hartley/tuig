//! This module handles all of the output, both abstractions and implementations.
//!
//! If you want to add more implementations, you need to:
//! - Add the relevant implementation of `Screen`, in a new submodule
//! - Modify `Screen::get` to properly detect and handle the new screen type, with `cfg!` or runtime checks

use core::{
    iter::FusedIterator,
    ops::{self, Range},
};

use alloc::vec::Vec;

use crate::{fmt::{Cell, FormattedExt, Text}, xy::XY};

/// An iterator over the rows of cells in a [`Screen`].
pub struct ScreenRows<'s> {
    screen: &'s Screen,
    rem: Range<usize>,
}

impl<'s> ScreenRows<'s> {
    fn new(screen: &'s Screen) -> Self {
        Self {
            screen,
            rem: 0..screen.size.y(),
        }
    }
    fn abs_row(&self, row: usize) -> &'s [Cell] {
        let start = row * self.screen.size.x();
        let end = start + self.screen.size.x();
        &self.screen.cells[start..end]
    }
}
macro_rules! fn_from_range {
    ( $( $fn:ident(
        $(& $(@@$ref:ident)?)? $(mut $(@@$mut:ident)?)? self
        $(, $args:ident: $type:ty)* $(,)?)
    ),* $(,)? ) => { $(
        fn $fn(
            $(& $($ref)?)? $(mut $($mut)?)? self
            $(, $args: $type)*
        ) -> Option<Self::Item> {
            let screen = self.screen;
            let row = self.rem.$fn($($args),*)?;
            Some(&screen[row])
        }
    )* }
}
impl<'s> Iterator for ScreenRows<'s> {
    type Item = &'s [Cell];
    fn_from_range! {
        next(&mut self), nth(&mut self, n: usize), last(self),
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.rem.size_hint()
    }
}
impl<'s> DoubleEndedIterator for ScreenRows<'s> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let row = self.rem.next_back()?;
        Some(self.abs_row(row))
    }
}
impl<'s> ExactSizeIterator for ScreenRows<'s> {}
impl<'s> FusedIterator for ScreenRows<'s> {}

/// A text framebuffer.
///
/// Allows you to render things onto it, then can be rendered onto the screen. This strategy avoids flickering,
/// partial renders, etc. and helps deduplicate rendering effort.
pub struct Screen {
    cells: Vec<Cell>,
    size: XY,
}

impl Screen {
    /// Create a new `Screen` in the given size.
    pub fn new(size: XY) -> Self {
        let mut res = Self {
            cells: alloc::vec![],
            size: XY(0, 0),
        };
        res.resize(size);
        res
    }

    /// How big this Screen is, in characters.
    pub fn size(&self) -> XY {
        self.size
    }

    /// All of the cells of this screen, in rows.
    ///
    /// i.e. for the screen:
    ///
    /// ```text
    /// 1 2
    /// 3 4
    /// ```
    ///
    /// this will return `&[1, 2, 3, 4]
    pub fn cells(&self) -> &[Cell] {
        &self.cells
    }

    /// Returns an iterator over the rows in a screen.
    pub fn rows(&self) -> ScreenRows {
        ScreenRows::new(self)
    }

    /// Clear this screen's contents, resetting it to the default and filling it with blank cells.
    pub fn clear(&mut self) {
        self.resize(self.size())
    }

    /// Resize the screen, clearing its contents at the same time. Does not reallocate unless the screen is growing.
    pub fn resize(&mut self, size: XY) {
        self.cells.truncate(0);
        self.cells.resize(size.x() * size.y(), Cell::BLANK);
        self.size = size;
    }

    /// Write some formatted text to the position on screen.
    ///
    /// This **does not** handle newlines or anything else. If you want that, see [`Textbox`].
    pub fn write(&mut self, pos: XY, text: Vec<Text>) {
        let XY(mut x, y) = pos;
        for chunk in text {
            for char in chunk.text.chars() {
                self[y][x] = Cell::of(char).fmt_of(&chunk);
                x += 1;
            }
        }
    }

    /// Write a header to the screen. (Note this must be rewritten every frame!)
    #[cfg(feature = "ui")]
    pub fn header<'a>(&'a mut self) -> crate::ui::widgets::Header<'a> {
        crate::ui::widgets::Header::new(self)
    }

    /// Write a text-box to the screen.
    #[cfg(feature = "ui")]
    pub fn textbox<'a>(&'a mut self, text: Vec<Text>) -> crate::ui::widgets::Textbox<'a> {
        crate::ui::widgets::Textbox::new(self, text)
    }

    /// Draw a vertical line on screen.
    #[cfg(feature = "ui")]
    pub fn vertical<'a>(&'a mut self, col: usize) -> crate::ui::widgets::Vertical<'a> {
        crate::ui::widgets::Vertical::new(self, col)
    }

    /// Draw a horizontal line on screen.
    #[cfg(feature = "ui")]
    pub fn horizontal<'a>(&'a mut self, row: usize) -> crate::ui::widgets::Horizontal<'a> {
        crate::ui::widgets::Horizontal::new(self, row)
    }
}

impl ops::Index<usize> for Screen {
    type Output = [Cell];
    fn index(&self, row: usize) -> &Self::Output {
        let start = row * self.size.x();
        let end = start + self.size.x();
        &self.cells[start..end]
    }
}

impl ops::IndexMut<usize> for Screen {
    fn index_mut(&mut self, row: usize) -> &mut Self::Output {
        let start = row * self.size.x();
        let end = start + self.size.x();
        &mut self.cells[start..end]
    }
}
