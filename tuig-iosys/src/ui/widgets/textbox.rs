use crate::{
    text, text1, screen::Screen, fmt::{Text, Formatted, FormattedExt}, xy::XY,
};

fn breakable(ch: char) -> bool {
    ch.is_whitespace()
}

/// Ancillary data which might be useful
#[derive(PartialEq, Eq, Clone)]
pub struct TextboxData {
    /// How many total lines there were, after word wrapping.
    pub lines: usize,
    /// How many lines on the screen the textbox occupied, accounting for the requested height, the
    pub height: usize,
    /// How far down from the top the displayed text started.
    pub scroll: usize,
}

impl TextboxData {
    const EMPTY: Self = Self {
        height: 0,
        lines: 0,
        scroll: 0,
    };
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
        let width = width.min(screen_size.x() - x);
        let height = self.height.unwrap_or(screen_size.y() - y);
        let mut height = height.min(screen_size.y() - y);
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
                    // tack on the end of the line, if it's not empty
                    if !line_end.is_empty() {
                        let rem_space = width - (pos + line_end.len());
                        line.push(chunk.with_text(line_end));
                        // then make sure the formatting continues into the next line
                        if rem_space > 0 {
                            line.push(text1!("{0:1$}"("", rem_space)).bg(chunk.get_fmt().bg));
                        }
                    }
                    // actually terminate the line and start the next one
                    lines.push(line);
                    line = text!["{0:1$}"("", self.indent)];
                    pos = self.indent;
                    line_start = true;
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
            height = real_height;
        } else {
            // we want [height] lines, starting [scroll] away from the top
            start = self.scroll;
        };

        let mut data = TextboxData {
            lines: lines.len(),
            height: 0,
            scroll: start,
        };
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
        match self.screen {
            Some(_) => {
                // this textbox hasn't been rendered, so do that now
                // (this dummy textbox has 0 allocations and should trigger a NOP rendering/drop)
                let dummy = Textbox {
                    screen: None,
                    chunks: vec![],
                    pos: XY(0, 0),
                    width: None,
                    height: None,
                    scroll: 0,
                    scroll_bottom: false,
                    indent: 0,
                    first_indent: None,
                };
                let me = std::mem::replace(self, dummy);
                // ignore the data
                let _ = me.render();
            }
            None => (),
        }
    }
}

#[cfg(test)]
mod test {
    use std::ops::{Bound, RangeBounds};

    use crate::{
        fmt::{Cell, Color, FormattedExt, Formatted},
        text1,
    };

    use super::*;

    const FILLER: &str = "0123456789abcdef";

    fn charat(x: usize, y: usize) -> char {
        // compare:
        // [(...) % FILLER.len()]
        // .chars().nth((...) % FILLER.len()).unwrap()
        // unfortunately the first one is invalid
        FILLER.chars().nth((x * 5 + y * 3) % FILLER.len()).unwrap()
    }

    /// Generate a screen filled with miscellaneous "random" data, to fairly reliably check that stuff was left blank
    /// by the code under test. (This isn't cryptographically secure or anything!)
    fn screen(x: usize, y: usize) -> Screen {
        let mut s = Screen::new(XY(x, y));
        for px in 0..x {
            for py in 0..y {
                s[py][px] = Cell::of(charat(px, py)).on_black();
            }
        }
        s
    }

    fn assert_cell_blank(s: &Screen, x: usize, y: usize) {
        let cell = &s[y][x];
        assert!(
            cell.ch == charat(x, y) && cell.get_fmt().bg == Color::Black,
            "mismatched cell at {}, {}: expected blank, got {:?}",
            x,
            y,
            cell,
        );
    }

    fn assert_area_blank(s: &Screen, x: impl RangeBounds<usize>, y: impl RangeBounds<usize>) {
        fn min(r: &impl RangeBounds<usize>) -> usize {
            match r.start_bound() {
                Bound::Included(v) => *v,
                Bound::Excluded(v) => v - 1,
                Bound::Unbounded => 0,
            }
        }
        fn max(r: &impl RangeBounds<usize>, m: usize) -> usize {
            match r.end_bound() {
                Bound::Included(v) => v + 1,
                Bound::Excluded(v) => *v,
                Bound::Unbounded => m,
            }
        }
        let min_x = min(&x);
        let max_x = max(&x, s.size().x());
        let min_y = min(&y);
        let max_y = max(&y, s.size().y());
        for x in min_x..max_x {
            for y in min_y..max_y {
                assert_cell_blank(s, x, y)
            }
        }
    }

    fn assert_cell_fmt(s: &Screen, x: usize, y: usize, c: Cell) {
        assert!(
            s[y][x] == c,
            "mismatched cell at {}, {}: expected {:?}, got {:?}",
            x,
            y,
            c,
            s[y][x]
        );
    }

    fn assert_area_fmt(s: &Screen, x: usize, y: usize, t: Text) {
        for (i, ch) in t.text.chars().enumerate() {
            assert_cell_fmt(s, x + i, y, Cell::of(ch).fmt_of(&t));
        }
    }

    macro_rules! screen_assert {
        ( $sc:ident: $(
            $( fmt $fmt_x:expr, $fmt_y:expr, $text:literal $( $mod:ident )* )?
            $( blank $blank_x:expr, $blank_y:expr )?
        ),* ) => {
            $(
                $(
                    assert_area_fmt(&$sc, $fmt_x, $fmt_y, text1!($($mod)* $text));
                )?
                $(
                    assert_area_blank(&$sc, $blank_x, $blank_y);
                )?
            )*
        };
    }

    #[test]
    fn blank_textbox_renders_nothing() {
        let mut sc = screen(50, 30);
        sc.textbox(vec![]);
        screen_assert!(sc: blank.., ..);
    }

    #[test]
    fn basic_textbox_renders_right() {
        let mut sc = screen(50, 30);
        let res = sc
            .textbox(text!("bleh ", red "blah ", green underline "bluh ", blue on_magenta "bloh "))
            .render();
        screen_assert!(sc:
            // end of the line and beyond
            blank 20.., 1..=1,
            // rest of the lines
            blank 1.., 2..,
            // and the formatted text
            fmt 0, 0, "bleh ",
            fmt 5, 0, "blah " red,
            fmt 10, 0, "bluh " green underline,
            fmt 15, 0, "bloh " blue on_magenta,
        );
        assert_eq!(res.height, 1);
        assert_eq!(res.lines, 1);
        assert_eq!(res.scroll, 0);
    }

    #[test]
    fn textbox_positioning_works() {
        let mut sc = screen(50, 30);
        let res = sc
            .textbox(text!("bleh ", red "blah ", green underline "bluh ", blue on_magenta "bloh "))
            .pos(4, 3)
            .render();
        screen_assert!(sc:
            // blank top 3 rows (0, 1, 2)
            blank .., ..3,
            // blank rows after 4
            blank .., 4..,
            // blank 0, 1, 2, 3 in row 3
            blank ..4, 3..=3,
            // blank rest of the line
            blank 24.., 3..=3,
            // then the formatted text
            fmt 4, 3, "bleh ",
            fmt 9, 3, "blah " red,
            fmt 14, 3, "bluh " green underline,
            fmt 19, 3, "bloh " blue on_magenta,
        );
        assert_eq!(res.height, 1);
        assert_eq!(res.lines, 1);
        assert_eq!(res.scroll, 0);
    }

    #[test]
    fn textbox_wraps_words_and_overwrites() {
        let mut sc = screen(50, 30);
        let res = sc
            .textbox(text!(
                "these are some words which will eveeeentually be wrapped!"
            ))
            .pos(40, 0)
            .render();
        screen_assert!(sc:
            blank ..40, ..,
            blank .., 6..,
            fmt 40, 0, "these are ",
            fmt 40, 1, "some words",
            fmt 40, 2, "which will",
            fmt 40, 3, "eveeeentu-",
            fmt 40, 4, "ally be   ",
            fmt 40, 5, "wrapped!", // last line isn't filled in!
        );
        assert_eq!(res.height, 6);
        assert_eq!(res.lines, 6);
        assert_eq!(res.scroll, 0);
    }

    #[test]
    fn textbox_wrap_carries_formatting() {
        let mut sc = screen(50, 30);
        sc.textbox(
            text!("these are some words which will ", green "eveeeentually", " be wrapped!"),
        )
        .pos(40, 0);
        screen_assert!(sc:
            blank ..40, ..,
            blank .., 6..,
            fmt 40, 0, "these are ",
            fmt 40, 1, "some words",
            fmt 40, 2, "which will",
            fmt 40, 3, "eveeeentu-" green,
            fmt 40, 4, "ally" green, fmt 44, 4, " be   ",
            fmt 40, 5, "wrapped!",
        );
    }

    #[test]
    fn textbox_linefill_carries_formatting() {
        let mut sc = screen(50, 30);
        sc.textbox(
            text!("these are some words which will eveeeentually ", on_blue "be wrapped", "!"),
        )
        .pos(40, 0);
        screen_assert!(sc:
            blank ..40, ..,
            blank .., 6..,
            fmt 40, 0, "these are ",
            fmt 40, 1, "some words",
            fmt 40, 2, "which will",
            fmt 40, 3, "eveeeentu-",
            fmt 40, 4, "ally ", fmt 45, 4, "be   " on_blue,
            fmt 40, 5, "wrapped" on_blue, fmt 47, 5, "!",
        );
    }

    #[test]
    fn textbox_size_truncates() {
        let mut sc = screen(50, 30);
        let res = sc
            .textbox(text!(
                "these are some words which will eveeeentually be wrapped!"
            ))
            .pos(40, 0)
            .size(10, 3)
            .render();
        screen_assert!(sc:
            blank ..40, ..,
            blank .., 3..,
            fmt 40, 0, "these are ",
            fmt 40, 1, "some words",
            fmt 40, 2, "which will",
        );
        assert_eq!(res.height, 3);
        assert_eq!(res.lines, 6);
        assert_eq!(res.scroll, 0);
    }

    #[test]
    fn textbox_scroll_moves_view() {
        let mut sc = screen(50, 30);
        let res = sc
            .textbox(text!(
                "these are some words which will eveeeentually be wrapped!"
            ))
            .pos(40, 0)
            .scroll(2)
            .render();
        screen_assert!(sc:
            blank ..40, ..,
            blank .., 4..,
            fmt 40, 0, "which will",
            fmt 40, 1, "eveeeentu-",
            fmt 40, 2, "ally be   ",
            fmt 40, 3, "wrapped!",
        );
        assert_eq!(res.height, 4);
        assert_eq!(res.lines, 6);
        assert_eq!(res.scroll, 2);
    }

    #[test]
    fn textbox_scroll_bottom_moves_view() {
        let mut sc = screen(50, 30);
        let res = sc
            .textbox(text!(
                "these are some words which will eveeeentually be wrapped!"
            ))
            .pos(40, 0)
            .size(10, 4)
            .scroll(1)
            .scroll_bottom(true)
            .render();
        screen_assert!(sc:
            blank ..40, ..,
            blank .., 4..,
            fmt 40, 0, "some words",
            fmt 40, 1, "which will",
            fmt 40, 2, "eveeeentu-",
            fmt 40, 3, "ally be   ",
        );
        assert_eq!(res.height, 4);
        assert_eq!(res.lines, 6);
        assert_eq!(res.scroll, 1);
    }
}
