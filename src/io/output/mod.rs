//! This module handles all of the output, both abstractions and implementations.
//!
//! If you want to add more implementations, you need to:
//! - Add the relevant implementation of `Screen`, in a new submodule
//! - Modify `Screen::get` to properly detect and handle the new screen type, with `cfg!` or runtime checks

mod text;
use std::ops;

pub use text::*;
mod widgets;
pub use widgets::*;

use super::XY;

#[derive(Clone, Copy)]
pub struct Cell {
    pub ch: char,
    pub fg: Color,
    pub bg: Color,
    pub bold: bool,
    pub underline: bool,
    pub invert: bool,
}

impl Cell {
    const BLANK: Self = Self {
        ch: ' ',
        fg: Color::White,
        bg: Color::Black,
        bold: false,
        underline: false,
        invert: false,
    };

    fn plain(ch: char) -> Self {
        Self { ch, ..Self::BLANK }
    }
}

/// A render target
pub struct Screen {
    cells: Vec<Cell>,
    size: XY,
}

impl Screen {
    pub fn new(size: XY) -> Self {
        let mut res = Self { cells: vec![], size: XY(0, 0) };
        res.resize(size);
        res
    }

    pub fn size(&self) -> XY {
        self.size
    }

    pub fn cells(&self) -> &[Cell] {
        &self.cells
    }

    pub fn rows(&self) -> Vec<&[Cell]> {
        let mut res = Vec::with_capacity(self.size.y());
        for y in 0..self.size.y() {
            res.push(&self[y]);
        }
        res
    }

    pub fn clear(&mut self) {
        self.resize(self.size())
    }

    pub fn resize(&mut self, size: XY) {
        self.cells.truncate(0);
        self.cells.resize(size.x() * size.y(), Cell::BLANK);
        self.size = size;
    }

    pub fn write(&mut self, pos: XY, text: Vec<Text>) {
        let XY(mut x, y) = pos;
        for chunk in text {
            for char in chunk.text.chars() {
                self[y][x] = Cell {
                    ch: char,
                    fg: chunk.fg,
                    bg: chunk.bg,
                    bold: chunk.bold,
                    underline: chunk.underline,
                    invert: chunk.invert,
                };
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

    pub fn vertical<'a>(&'a mut self, col: usize) -> Vertical<'a> {
        Vertical::new(self, col)
    }

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
