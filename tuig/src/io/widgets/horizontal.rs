use crate::{
    cell,
    io::{output::Screen
    }, format::{Cell, Color, FormattedExt},
};

/// A horizontal line across all or part of the screen
pub struct Horizontal<'a> {
    pub(in super::super) screen: &'a mut Screen,
    pub(in super::super) row: usize,
    pub(in super::super) start: Option<usize>,
    pub(in super::super) end: Option<usize>,
    pub(in super::super) fill: Cell,
}

impl<'a> Horizontal<'a> {
    pub fn new(screen: &'a mut Screen, row: usize) -> Self {
        Horizontal {
            screen,
            row,
            start: None,
            end: None,
            fill: cell!('-'),
        }
    }

    crate::util::setters! {
        start(x: usize) => start = Some(x),
        end(x: usize) => end = Some(x),
        fill(c: Cell) => fill = c,
    }

    pub fn color(mut self, c: Color) -> Self {
        let new_fill = self.fill.clone().fg(c);
        self.fill = new_fill;
        self
    }
}

impl<'a> Drop for Horizontal<'a> {
    fn drop(&mut self) {
        let start_x = self.start.unwrap_or(0);
        let end_x = self.end.unwrap_or(self.screen.size().x());
        for x in start_x..end_x {
            self.screen[self.row][x] = self.fill.clone();
        }
    }
}
