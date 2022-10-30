use crate::{io::{output::{Screen, Text}, XY}, text};

fn breakable(ch: char) -> bool {
    // TODO: There have to be better ways to decide breakability than just "nyehe whitespaec"
    ch.is_whitespace()
}

/// A box of text which can be written to a `Screen`. Note these are meant to be generated on the fly, every frame,
/// possibly multiple times. They do the actual *writing* when they're dropped, converting the higher-level Textbox
/// API things to calls of `Screen::raw`.
pub struct Textbox<'a> {
    pub(in super::super) screen: &'a mut Screen,
    pub(in super::super) chunks: Vec<Text>,
    pub(in super::super) pos: XY,
    pub(in super::super) width: Option<usize>,
    pub(in super::super) height: Option<usize>,
    pub(in super::super) scroll: usize,
    pub(in super::super) scroll_bottom: bool,
    pub(in super::super) indent: usize,
    pub(in super::super) first_indent: Option<usize>,
}

impl<'a> Textbox<'a> {
    pub fn new(screen: &'a mut Screen, text: Vec<Text>) -> Self {
        Self {
            screen,
            chunks: text,
            pos: XY(0, 0),
            width: None,
            height: None,
            scroll: 0,
            scroll_bottom: false,
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
        scroll_bottom(v: bool) => scroll_bottom = v,
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
    if scroll_bottom != false,
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

        // break the chunks into paragraphs on newlines
        let mut paragraphs = vec![];
        let mut cur_para = vec![];
        for mut chunk in std::mem::replace(&mut self.chunks, vec![]) {
            while let Some((line, rest)) = chunk.text.split_once('\n') {
                cur_para.push(chunk.with_text(line.into()));
                paragraphs.push(cur_para);
                cur_para = vec![];
                chunk.text = rest.into();
            }
            cur_para.push(chunk);
        }
        paragraphs.push(cur_para);

        // space out and word-wrap those paragraphs into lines
        let mut lines = vec![];
        for para in paragraphs {
            let mut line: Vec<Text> = text!["{0:1$}"("", first_indent)];
            let mut pos = first_indent;
            let mut line_start = true;
            for mut chunk in para {
                // the code flow in this for loop is too complex to add this =false at the end, so we make do
                let was_line_start = line_start;
                line_start = false;
                // while there's too much to fit on the next line all at once
                while pos + chunk.text.len() >= width {
                    // how much space can we fit things into?
                    let space_left = width - pos - 1;
                    // the bit of text that will be put at the end of this line
                    let line_end: String;
                    // the rest of the text, which wraps to following lines
                    let rest: String;
                    if let Some(idx) = chunk.text[..space_left].rfind(breakable) {
                        // we have a breakable character in time; we break there
                        let pre = &chunk.text[..idx];
                        let post = &chunk.text[idx+1..];
                        line_end = pre.into();
                        rest = post.into();
                    } else if !was_line_start {
                        // no breakable character, but we're not at the start of the line, so let's try
                        // ending the line here and getting to the next one
                        line_end = String::new();
                        rest = chunk.text;
                    } else if space_left > 1 {
                        // break the word with a hyphen, since there's space for it
                        let (pre, post) = chunk.text.split_at(space_left - 1);
                        line_end = format!("{}-", pre);
                        rest = post.into();
                    } else if space_left == 1 {
                        // no room for a hyphen, so just pull one letter off
                        let (pre, post) = chunk.text.split_at(1);
                        line_end = pre.into();
                        rest = post.into();
                    } else {
                        // at the start of a line, but 0 space left -- the asserts above should have
                        // prevented this!
                        unreachable!("indent or first indent is larger than width")
                    }
                    // set up the chunk for next iteration
                    chunk.text = rest;
                    // tack on the end of the line
                    if !line_end.is_empty() {
                        // (avoiding the allocation if necessary)
                        line.push(chunk.with_text(line_end));
                    }
                    // actually terminate the line and start the next one
                    lines.push(line);
                    line = text!["{0:1$}"("", self.indent)];
                    pos = self.indent;
                }
                // now we can fit the rest on this one line
                pos += chunk.text.len();
                line.push(chunk);
            }
            lines.push(line);
        }

        let x = self.pos.x();
        let mut y = self.pos.y();
        let start;
        if self.scroll_bottom {
            // we want [height] lines, starting [scroll] away from the bottom
            let end = lines.len() - self.scroll;
            start = end - height;
        } else {
            // we want [height] lines, starting [scroll] away from the top
            start = self.scroll;
        };
        for line in lines.into_iter().skip(start).take(height) {
            self.screen.write(XY(x, y), line);
            y += 1;
        }
    }
}
