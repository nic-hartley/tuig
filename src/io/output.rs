//! This module handles all of the output, both abstractions and implementations.
//!
//! If you want to add more implementations, you need to:
//! - Add the relevant implementation of `Screen`, in a new submodule
//! - Modify `Screen::get` to properly detect and handle the new screen type, with `cfg!` or runtime checks

use std::ops;

pub use super::clifmt::*;
pub use super::widgets::*;

use super::XY;

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
            cells: vec![],
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

    /// The rows of cells of this `Screen`.
    pub fn rows(&self) -> Vec<&[Cell]> {
        let mut res = Vec::with_capacity(self.size.y());
        for y in 0..self.size.y() {
            res.push(&self[y]);
        }
        res
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
    pub fn header<'a>(&'a mut self) -> Header<'a> {
        Header::new(self)
    }

    /// Write a text-box to the screen.
    pub fn textbox<'a>(&'a mut self, text: Vec<Text>) -> Textbox<'a> {
        Textbox::new(self, text)
    }

    /// Draw a vertical line on screen.
    pub fn vertical<'a>(&'a mut self, col: usize) -> Vertical<'a> {
        Vertical::new(self, col)
    }

    /// Draw a horizontal line on screen.
    pub fn horizontal<'a>(&'a mut self, row: usize) -> Horizontal<'a> {
        Horizontal::new(self, row)
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
