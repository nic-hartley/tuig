use core::mem;

use alloc::{string::String, vec::Vec};

use crate::{
    fmt::{Cell, Formatted, FormattedExt, Text},
    text, text1,
    xy::XY, ui::{ScreenView, RawAttachment},
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

/// A box of text which can be attached to a [`Region`].
///
/// Textboxes automatically handle:
///
/// - Word wrapping to fit in their region
/// - Indentation, including distinct first line indentation
/// - Scrolling to a desired height, relative to the top or bottom
pub struct Textbox {
    pub(in super::super) chunks: Vec<Text>,
    pub(in super::super) scroll: usize,
    pub(in super::super) scroll_bottom: bool,
    pub(in super::super) indent: usize,
    pub(in super::super) first_indent: Option<usize>,
}

impl Textbox {
    /// Create a new textbox containing the given text.
    pub fn new(text: Vec<Text>) -> Self {
        Self {
            chunks: text,
            scroll: 0,
            scroll_bottom: false,
            indent: 0,
            first_indent: None,
        }
    }

    crate::util::setters! {
        /// Set the scroll position of the textbox, i.e. how many lines from the top or bottom should be hidden.
        /// 
        /// Defaults to 0, i.e. not scrolling at all. Anything that doesn't fit is simply not visible.
        scroll(amt: usize) => scroll = amt,
        /// Set whether the scroll position should be relative to the top or bottom.
        /// 
        /// Scrolling from the bottom will also align the bottom of the text with the bottom of the textbox, rather
        /// than aligning the tops.
        /// 
        /// Defaults to false, i.e. by default scrolling is from the top.
        scroll_bottom(v: bool) => scroll_bottom = v,
        /// How much to indent the text.
        /// 
        /// Defaults to 0, i.e. no indent.
        indent(amt: usize) => indent = amt,
        /// How much to specifically indent the first line of each paragraph.
        /// 
        /// Defaults to being the same as the indent.
        first_indent(amt: usize) => first_indent = Some(amt),
    }

    /// Render this textbox to a [`ScreenView`], and return information about the render.
    /// 
    /// This is functionally equivalent to just directly [`Region::attach`][crate::ui::Region::attach]ing the textbox,
    /// but you may find it useful if e.g. you want the text to depend on the input being handled in that region.
    pub fn render_to(mut self, mut sv: ScreenView) -> TextboxData {
        if sv.size() == XY(0, 0) {
            return TextboxData::EMPTY;
        }

        let first_indent = self.first_indent.unwrap_or(self.indent);
        let width = sv.size().x();
        let mut height = sv.size().y();

        assert!(width > self.indent);
        assert!(width > first_indent);

        // break the chunks into paragraphs on newlines
        let mut paragraphs = alloc::vec![];
        let mut cur_para = alloc::vec![];
        for mut chunk in mem::replace(&mut self.chunks, alloc::vec![]) {
            while let Some((line, rest)) = chunk.text.split_once('\n') {
                cur_para.push(chunk.with_text(line.into()));
                paragraphs.push(cur_para);
                cur_para = alloc::vec![];
                chunk.text = rest.into();
            }
            if !chunk.text.is_empty() {
                cur_para.push(chunk);
            }
        }
        paragraphs.push(cur_para);

        // space out and word-wrap those paragraphs into lines
        let mut lines = alloc::vec![];
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
                        line_end = alloc::format!("{}-", pre);
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

        let x = 0;
        let mut y = 0;

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
        let mut cells = alloc::vec![];
        for line in lines.into_iter().skip(start).take(height) {
            cells.extend(
                line.iter()
                    .flat_map(|t| t.text.chars().map(|c| Cell::of(c).fmt_of(t))),
            );
            sv[y][x..x + cells.len()].clone_from_slice(&cells);
            cells.clear();
            y += 1;
            data.height += 1;
        }
        data
    }
}

impl<'s> RawAttachment<'s> for Textbox {
    type Output = TextboxData;
    fn raw_attach(self, _: crate::Action, screen: ScreenView<'s>) -> Self::Output {
        self.render_to(screen)
    }
}

#[cfg(test)]
mod test {
    use core::ops::{Bound, RangeBounds};

    use crate::{
        fmt::{Cell, Color, Formatted, FormattedExt},
        ui::Region,
        Screen,
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

    /// Generate a screen filled with miscellaneous "random" data, to fairly reliably check that stuff was left alone
    /// by the code under test, and offer a region of the given size and position within it.
    macro_rules! make_screen {
        (
            $screen:ident($sx:literal, $sy:literal)
            $( , $region:ident($rx:literal, $ry:literal, $rw:tt, $rh:tt) )?
        ) => {
            let mut $screen = Screen::new(XY($sx, $sy));
            for px in 0..$sx {
                for py in 0..$sy {
                    $screen[py][px] = Cell::of(charat(px, py)).on_black();
                }
            }
            $(
                let root = Region::new(&mut $screen, crate::Action::Redraw);
                let [_, vert] = root.split(crate::ui::cols!($rx $rw))
                    .expect("not enough space for desired cols");
                let [_, hori] = vert.split(crate::ui::rows!($ry $rh))
                    .expect("not enough space for desired rows");
                #[allow(unused_mut)]
                let mut $region = hori;
            )?
        };
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
        make_screen!(sc(50, 30), r(0, 0, *, *));
        r.textbox(alloc::vec![]);
        screen_assert! { sc: blank .., .. };
    }

    #[test]
    fn basic_textbox_renders_right() {
        make_screen!(sc(50, 30), r(0, 0, *, *));
        let res = r
            .textbox(text!("bleh ", red "blah ", green underline "bluh ", blue on_magenta "bloh "));
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
        make_screen!(sc(50, 30), r(4, 3, *, *));
        let res = r
            .textbox(text!("bleh ", red "blah ", green underline "bluh ", blue on_magenta "bloh "));
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
        make_screen!(sc(50, 30), r(40, 0, *, *));
        let res = r
            .textbox(text!(
                "these are some words which will eveeeentually be wrapped!"
            ));
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
        make_screen!(sc(50, 30), r(40, 0, *, *));
        r.textbox(
            text!("these are some words which will ", green "eveeeentually", " be wrapped!"),
        );
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
        make_screen!(sc(50, 30), r(40, 0, *, *));
        r.textbox(
            text!("these are some words which will eveeeentually ", on_blue "be wrapped", "!"),
        );
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
        make_screen!(sc(50, 30), r(40, 0, 10, 3));
        let res = r
            .textbox(text!(
                "these are some words which will eveeeentually be wrapped!"
            ));
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
        make_screen!(sc(50, 30), r(40, 0, *, *));
        let res = r.attach(
            Textbox::new(text!(
                "these are some words which will eveeeentually be wrapped!"
            )).scroll(2)
        );
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
        make_screen!(sc(50, 30), r(40, 0, 10, 4));
        let res = r.attach(
            Textbox::new(text!(
                "these are some words which will eveeeentually be wrapped!"
            ))
            .scroll(1)
            .scroll_bottom(true)
        );
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
