use crate::io::{output::{Cell, Screen}, clifmt::Color};

pub struct Vertical<'a> {
    pub(in super::super) screen: &'a mut Screen,
    pub(in super::super) col: usize,
    pub(in super::super) start: Option<usize>,
    pub(in super::super) end: Option<usize>,
    pub(in super::super) char: char,
    pub(in super::super) fg: Color,
    pub(in super::super) bg: Color,
}

impl<'a> Vertical<'a> {
    pub fn new(screen: &'a mut Screen, col: usize) -> Self {
        Vertical {
            screen,
            col,
            start: None,
            end: None,
            char: '|',
            fg: Color::Default,
            bg: Color::Default,
        }
    }

    crate::util::setters! {
        start(y: usize) => start = Some(y),
        end(y: usize) => end = Some(y),
        char(ch: char) => char = ch,
        fg(c: Color) => fg = c,
        bg(c: Color) => bg = c,
    }
}

crate::util::abbrev_debug! {
    Vertical<'a>;
    write col,
    if start != None,
    if end != None,
    if char != '|',
    if fg != Color::Default,
    if bg != Color::Default,
}

impl<'a> Drop for Vertical<'a> {
    fn drop(&mut self) {
        let start_y = self.start.unwrap_or(0);
        let end_y = self.end.unwrap_or(self.screen.size().y());
        for y in start_y..end_y {
            self.screen[y][self.col] = Cell {
                ch: self.char,
                fg: self.fg,
                bg: self.bg,
                ..Cell::BLANK
            };
        }
    }
}
