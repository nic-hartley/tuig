use crate::io::{output::{Cell, Screen}, text::Color};

pub struct Horizontal<'a> {
    pub(in super::super) screen: &'a mut Screen,
    pub(in super::super) row: usize,
    pub(in super::super) start: Option<usize>,
    pub(in super::super) end: Option<usize>,
    pub(in super::super) char: char,
    pub(in super::super) fg: Color,
    pub(in super::super) bg: Color,
}

impl<'a> Horizontal<'a> {
    pub fn new(screen: &'a mut Screen, row: usize) -> Self {
        Horizontal {
            screen,
            row,
            start: None,
            end: None,
            char: '-',
            fg: Color::Default,
            bg: Color::Default,
        }
    }

    crate::util::setters! {
        start(x: usize) => start = Some(x),
        end(x: usize) => end = Some(x),
        char(ch: char) => char = ch,
        fg(c: Color) => fg = c,
        bg(c: Color) => bg = c,
    }
}

crate::util::abbrev_debug! {
    Horizontal<'a>;
    write row,
    if start != None,
    if end != None,
    if char != '-',
    if fg != Color::Default,
    if bg != Color::Default,
}

impl<'a> Drop for Horizontal<'a> {
    fn drop(&mut self) {
        let start_x = self.start.unwrap_or(0);
        let end_x = self.end.unwrap_or(self.screen.size().x());
        for x in start_x..end_x {
            self.screen[self.row][x] = Cell {
                ch: self.char,
                fg: self.fg,
                bg: self.bg,
                ..Cell::BLANK
            };
        }
    }
}
