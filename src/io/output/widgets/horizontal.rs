use crate::io::{output::{Text, Screen}, XY};

pub struct Horizontal<'a> {
    pub(in super::super) screen: &'a mut dyn Screen,
    pub(in super::super) row: usize,
    pub(in super::super) start: Option<usize>,
    pub(in super::super) end: Option<usize>,
    pub(in super::super) char: char,
}

impl<'a> Horizontal<'a> {
    crate::util::setters! {
        start(x: usize) => start = Some(x),
        end(x: usize) => end = Some(x),
        char(ch: char) => char = ch,
    }
}

crate::util::abbrev_debug! {
    Horizontal<'a>;
    write row,
    if start != None,
    if end != None,
    if char != '-',
}

impl<'a> Drop for Horizontal<'a> {
    fn drop(&mut self) {
        let start_x = self.start.unwrap_or(0);
        let end_x = self.end.unwrap_or(self.screen.size().x());
        let text = self.char.to_string().repeat(end_x - start_x);
        self.screen.write_raw_single(Text::of(text), XY(start_x, self.row));
    }
}
