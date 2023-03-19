use crate::{cell, screen::Screen, fmt::Cell};

/// A vertical line across all or part of the screen
pub struct Vertical<'a> {
    pub(in super::super) screen: &'a mut Screen,
    pub(in super::super) col: usize,
    pub(in super::super) start: Option<usize>,
    pub(in super::super) end: Option<usize>,
    pub(in super::super) fill: Cell,
}

impl<'a> Vertical<'a> {
    pub fn new(screen: &'a mut Screen, col: usize) -> Self {
        Vertical {
            screen,
            col,
            start: None,
            end: None,
            fill: cell!('|'),
        }
    }

    crate::util::setters! {
        start(y: usize) => start = Some(y),
        end(y: usize) => end = Some(y),
    }
}

impl<'a> Drop for Vertical<'a> {
    fn drop(&mut self) {
        let start_y = self.start.unwrap_or(0);
        let end_y = self.end.unwrap_or(self.screen.size().y());
        for y in start_y..end_y {
            self.screen[y][self.col] = self.fill.clone();
        }
    }
}
