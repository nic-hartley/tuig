use crate::{
    io::{
        output::{Screen, Text},
        XY,
    },
    text,
};

fn breakable(ch: char) -> bool {
    // TODO: There have to be better ways to decide breakability than just "nyehe whitespaec"
    ch.is_whitespace()
}

/// Ancillary data which might be useful
pub struct TextboxData {
    /// How many total lines there were, after word wrapping.
    pub lines: usize,
    /// How many lines on the screen the textbox occupied, accounting for the requested height, the 
    pub height: usize,
    /// How far down from the top the displayed text started.
    pub scroll: usize,
}

impl TextboxData {
    const EMPTY: Self = Self { height: 0, lines: 0, scroll: 0 };
}

/// A box of text which can be written to a `Screen`. Note these are meant to be generated on the fly, every frame,
/// possibly multiple times. They do the actual *writing* when they're dropped, converting the higher-level Textbox
/// API things to calls of `Screen::raw`.
pub struct Textbox<'a> {
    pub(in super::super) screen: Option<&'a mut Screen>,
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
            screen: Some(screen),
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

    pub fn render(mut self) -> TextboxData {
        let screen = match std::mem::replace(&mut self.screen, None) {
            Some(s) => s,
            None => return TextboxData::EMPTY,
        };

        let first_indent = self.first_indent.unwrap_or(self.indent);
        let XY(x, y) = self.pos;

        let screen_size = screen.size();
        let width = self.width.unwrap_or(screen_size.x() - x);
        let height = self.height.unwrap_or(screen_size.y() - y);
        if width == 0 || height == 0 {
            // nothing to draw
            return TextboxData::EMPTY;
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
            if !chunk.text.is_empty() {
                cur_para.push(chunk);
            }
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
                while pos + chunk.text.len() > width {
                    // how much space can we fit things into?
                    let space_left = width - pos;
                    // the bit of text that will be put at the end of this line
                    let line_end: String;
                    // the rest of the text, which wraps to following lines
                    let rest: String;
                    if let Some(idx) = chunk.text[..space_left + 1].rfind(breakable) {
                        // we have a breakable character in time; we break there
                        let pre = &chunk.text[..idx];
                        let post = &chunk.text[idx + 1..];
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
            start = end.saturating_sub(height);
            // make sure the text fills from the bottom instead of the top
            let real_height = end - start;
            y += height - real_height;
        } else {
            // we want [height] lines, starting [scroll] away from the top
            start = self.scroll;
        };

        let mut data = TextboxData { lines: lines.len(), height: 0, scroll: start };
        for line in lines.into_iter().skip(start).take(height) {
            screen.write(XY(x, y), line);
            y += 1;
            data.height += 1;
        }
        data
    }
}

impl<'a> Drop for Textbox<'a> {
    fn drop(&mut self) {
        // should have 0 allocations to generate the dummy textbox, and the empty textbox should immediately quit
        // trying to render (assuming DCE doesn't notice the screen isn't used and delete it all anyway)
        let dummy = Textbox {
            screen: None, chunks: vec![], pos: XY(0, 0),
            width: None, height: None,
            scroll: 0, scroll_bottom: false,
            indent: 0, first_indent: None
        };
        let me = std::mem::replace(self, dummy);
        let _ = me.render();
    }
}
