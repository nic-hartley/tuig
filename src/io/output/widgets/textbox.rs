use crate::io::{output::{Screen, Text}, XY};

/// A box of text which can be written to a `Screen`. Note these are meant to be generated on the fly, every frame,
/// possibly multiple times. They do the actual *writing* when they're dropped, converting the higher-level Textbox
/// API things to calls of `Screen::raw`.
pub struct Textbox<'a> {
    pub(in super::super) screen: &'a mut dyn Screen,
    pub(in super::super) chunks: Vec<Text>,
    pub(in super::super) pos: XY,
    pub(in super::super) width: Option<usize>,
    pub(in super::super) height: Option<usize>,
    pub(in super::super) scroll: usize,
    pub(in super::super) indent: usize,
    pub(in super::super) first_indent: Option<usize>,
}

impl<'a> Textbox<'a> {
    pub fn new(screen: &'a mut dyn Screen, text: Vec<Text>) -> Self {
        Self {
            screen,
            chunks: text,
            pos: XY(0, 0),
            width: None,
            height: None,
            scroll: 0,
            indent: 0,
            first_indent: None,
        }
    }

    pub fn size(mut self, x: usize, y: usize) -> Self {
        self.width = Some(x);
        self.height = Some(y);
        self
    }

    crate::util::setters! {
        pos(x: usize, y: usize) => pos = XY(x, y),
        xy(xy: XY) => pos = xy,
        width(w: usize) => width = Some(w),
        height(h: usize) => height = Some(h),
        scroll(amt: usize) => scroll = amt,
        indent(amt: usize) => indent = amt,
        first_indent(amt: usize) => first_indent = Some(amt),
    }
}

crate::util::abbrev_debug! {
    Textbox<'a>;
    ignore chunks,
    if pos != XY(0, 0),
    if width != None,
    if height != None,
    if scroll != 0,
    if indent != 0,
    if first_indent != None,
}

impl<'a> Drop for Textbox<'a> {
    fn drop(&mut self) {
        let first_indent = self.first_indent.unwrap_or(self.indent);
        let XY(x, y) = self.pos;

        let screen_size = self.screen.size();
        let width = self.width.unwrap_or(screen_size.x() - x);
        let height = self.height.unwrap_or(screen_size.y() - y);
        if width == 0 || height == 0 {
            // nothing to draw
            return;
        }

        assert!(width > self.indent);
        assert!(width > first_indent);

        let mut col = first_indent;
        let mut line_num = 0;
        let mut line_start = true;

        macro_rules! write_raw {
            ($text:expr) => {
                if line_num >= self.scroll && line_num - self.scroll < height {
                    self.screen.write_raw(
                        $text,
                        XY(x + col, y + line_num - self.scroll)
                    );
                }
            }
        }

        macro_rules! next_line {
            ($new_para:expr) => {
                line_num += 1;
                line_start = true;
                col = if $new_para { first_indent } else { self.indent };
            }
        }

        macro_rules! do_wrap {
            ($chunk:ident, $line:ident) => {
                while $line.len() > width - col {
                    if let Some(break_pos) = $line[..width - col].rfind(char::is_whitespace) {
                        let (subline, rest_of_line) = $line.split_at(break_pos);
                        $line = rest_of_line.trim_start();

                        write_raw!(vec![$chunk.with_text(subline.into())]);
                    } else if line_start {
                        // if we're already at the start of the line, can't exactly push stuff to the next line;
                        // that'd loop forever
                        let rest_of_line = if width - col == 1 {
                            // we can't just take 0 characters, so ignore the hyphen if we only have space for a
                            // single character
                            let (subline, rest) = $line.split_at(1);
                            write_raw!(vec![$chunk.with_text(subline.to_string())]);
                            rest
                        } else {
                            // otherwise we have enough room for at least one character plus a hyphen
                            let (subline, rest) = $line.split_at(width - col - 1);
                            write_raw!(vec![$chunk.with_text(subline.to_string() + "-")]);
                            rest
                        };
                        $line = rest_of_line.trim_start();
                    } else {
                        // if we've just finished another chunk, so it's *not* the beginning of a line, then just go
                        // to the next line for anything that's too long
                        // (that means we don't do anything here; this branch is just for documentation)
                    }
                    next_line!(false);
                }
                if $line.len() > 0 {
                    write_raw!(vec![$chunk.with_text($line.into())]);
                    #[allow(unused_assignments)] {
                        col += $line.len();
                        line_start = false;
                    }
                }
            }
        }

        for chunk in &self.chunks {
            let mut rest = &chunk.text[..];
            while let Some(nl_pos) = rest.find('\n') {
                let (mut line, new_rest) = rest.split_at(nl_pos);
                rest = &new_rest[1..];

                do_wrap!(chunk, line);
                next_line!(true);
            }
            if !rest.is_empty() {
                do_wrap!(chunk, rest);
            } else {
                // ended with a newline, don't bother trying to format the zero remaining characters, that'll just
                // cause problems (so just pass on to the next line)
            }
        }
    }
}
