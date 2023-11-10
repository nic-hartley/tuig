use core::{iter, mem};

use alloc::{collections::VecDeque, string::String};
use tuig_iosys::{
    fmt::{Cell, Format, Formatted, FormattedExt, Text},
    text1, Action, Key,
};

use crate::ScreenView;

use super::RawAttachment;

/// Takes text input, analogous to `<input type="text">`, with hooks for autocompletion, history, etc.
///
/// `TextInput` is meant to take a single line of input, and it doesn't really do anything with extra vertical space
/// it's attached to. If the input is longer than the space is wide, the input scrolls left and right to keep the
/// cursor in view, biasing towards the end, and cutting off the ends with a `…`.
///
/// You'll need to keep the actual element around, because it tracks some pieces of state:
/// - Currently in-progress input
/// - Cursor position within the line
/// - Autocomplete-related things
///
/// Accordingly, [`Attachment`](super::Attachment) (through [`RawAttachment`]) is implemented for `&mut TextInput`,
/// not `TextInput` itself, so you'll use it like:
///
/// ```no_run
/// # use tuig_ui::{Region, attachments::{TextInput, TextInputResult}};
/// let region = //...
/// # Region::empty();
/// let mut text_input = // ...
/// # TextInput::new("", 0);
/// match region.attach(&mut text_input) {
///   TextInputResult::Nothing => (),
///   TextInputResult::Autocomplete { text, res } => *res = text.into(),
///   TextInputResult::Submit(s) => println!("enter pressed: {}", s),
/// }
/// ```
///
/// [`Region::attach`](super::Region::attach)ing this will return a [`TextInputResult`], which is how you'll interact
/// with autocomplete. To use the history features, see [`TextInput::store`].
pub struct TextInput {
    /// A bit of fixed, uneditable text at the beginning of the text input, to signal the user to type.
    pub prompt: String,

    /// The current line of text being edited
    pub line: String,
    /// Which character index the cursor is just before (so `cursor == line.len()` means the cursor is at the end)
    pub cursor: usize,

    /// The caller-specified autocomplete text
    pub autocomplete: String,

    /// Previous lines we were told to save
    ///
    /// The history goes older to newer from front to back, so new lines are added with `push_back` and old ones are
    /// removed with `pop_front`.
    pub history: VecDeque<String>,
    /// Current position, if scrolling back through history, or `history.len()` if looking at `line`
    pub histpos: usize,
    /// Maximum number of history elements
    pub histcap: usize,
}

impl TextInput {
    /// Creates a [`TextInput`] with the prompt text and history capacity.
    ///
    /// The prompt is the fixed, noneditable text at the beginning of the `TextInput`.
    ///
    /// `history_cap` is the maximum number of history entries. If it's 0, there's no history at all; otherwise, there
    /// will be at most that many history entries.
    pub fn new(prompt: impl Into<String>, history_cap: usize) -> Self {
        Self {
            prompt: prompt.into(),
            line: String::new(),
            cursor: 0,
            autocomplete: String::new(),
            history: VecDeque::new(),
            histpos: 0,
            histcap: history_cap,
        }
    }

    /// Store a line in the history, usually one you just got from [`TextInputResult::Submit`]. (But that isn't
    /// required or enforced.)
    ///
    /// The user can scroll through history with the up and down arrows. The `TextInput` will show the history entries
    /// in order. When the user types, it copies the selected history into the current line, rather than editing the
    /// history.
    pub fn store(&mut self, line: String) {
        if self.histcap == 0 {
            return;
        }
        if self.history.len() == self.histcap {
            // UNWRAP: histcap > 0, and len == histcap, so len > 0, so pop works
            let old = self.history.pop_front().unwrap();
            if self.line.capacity() == 0 {
                // haven't allocated the line yet, so replace it with an empty but allocated line
                self.line = old;
                self.line.clear();
            }
            self.history.push_back(line);
            // track the histpos backwards, if we're in the history
            if self.histpos < self.history.len() {
                self.histpos -= 1;
            }
        } else {
            if self.histpos == self.history.len() {
                self.histpos += 1;
            }
            self.history.push_back(line);
        }
    }

    fn cur_line(&self) -> &str {
        if self.histpos == self.history.len() {
            &self.line
        } else {
            &self.history[self.histpos]
        }
    }

    fn sel_line(&mut self) {
        if self.histpos < self.history.len() {
            self.line = self.history[self.histpos].clone();
            self.histpos = self.history.len();
        }
    }

    fn input(&mut self, input: Action) -> Option<TextInputResult<'static>> {
        match input {
            Action::KeyPress { key: Key::Char(ch) } => {
                self.sel_line();
                self.line.insert(self.cursor, ch);
                self.cursor += 1;
                self.autocomplete.clear();
                Some(TextInputResult::Nothing)
            }
            Action::KeyPress { key: Key::Home } => {
                self.cursor = 0;
                self.autocomplete.clear();
                Some(TextInputResult::Nothing)
            }
            Action::KeyPress { key: Key::End } => {
                self.cursor = self.line.len();
                self.autocomplete.clear();
                Some(TextInputResult::Nothing)
            }
            Action::KeyPress { key: Key::Left } => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                }
                self.autocomplete.clear();
                Some(TextInputResult::Nothing)
            }
            Action::KeyPress { key: Key::Right } => {
                if self.cursor < self.cur_line().len() {
                    self.cursor += 1;
                }
                self.autocomplete.clear();
                Some(TextInputResult::Nothing)
            }
            Action::KeyPress { key: Key::Up } => {
                if self.histpos > 0 {
                    self.histpos -= 1;
                    self.cursor = self.cur_line().len();
                }
                self.autocomplete.clear();
                Some(TextInputResult::Nothing)
            }
            Action::KeyPress { key: Key::Down } => {
                if self.histpos < self.history.len() {
                    self.histpos += 1;
                    self.cursor = self.cur_line().len();
                }
                self.autocomplete.clear();
                Some(TextInputResult::Nothing)
            }
            Action::KeyPress {
                key: Key::Backspace,
            } => {
                self.sel_line();
                if self.cursor > 0 {
                    self.cursor -= 1;
                    self.line.remove(self.cursor);
                }
                self.autocomplete.clear();
                Some(TextInputResult::Nothing)
            }
            Action::KeyPress { key: Key::Delete } => {
                self.sel_line();
                if self.cursor < self.line.len() {
                    self.line.remove(self.cursor);
                }
                self.autocomplete.clear();
                Some(TextInputResult::Nothing)
            }
            Action::KeyPress { key: Key::Tab } => {
                self.sel_line();
                self.autocomplete.clear();
                None
            }
            Action::KeyPress { key: Key::Enter } => {
                self.sel_line();
                self.cursor = 0;
                self.autocomplete.clear();
                Some(TextInputResult::Submit(mem::take(&mut self.line)))
            }
            _ => Some(TextInputResult::Nothing),
        }
    }

    fn render(&self, mut screen: ScreenView) {
        // TODO: Rewrite like. all of this once #32 lands. it's so bad,,,

        // calculate how wide the right should be
        let width = screen.size().x() - self.prompt.len();
        let min_space_left = usize::min(1 + width / 8, self.cursor);
        let max_space_right = width - min_space_left;
        let all_right = self.cur_line().len() - self.cursor + self.autocomplete.len();
        let (len_right, cut_right) = if all_right == 0 {
            (1, false)
        } else if all_right <= max_space_right {
            (all_right, false)
        } else {
            (max_space_right - 1, true)
        };

        // calculate left side space
        let max_space_left = width - (len_right + cut_right as usize);
        let all_left = self.cursor;
        let (len_left, cut_left) = if all_left <= max_space_left {
            (all_left, false)
        } else {
            (max_space_left - 1, true)
        };

        // 5 chunks: prompt, precursor, cursor, autocomplete, postcursor
        let mut line = alloc::vec::Vec::with_capacity(5);
        line.push(text1!("{}"(self.prompt)));
        line.push(text1!("{}"(&self.cur_line()[..self.cursor])));
        line.push(text1!("")); // cursor, eventually
        if !self.autocomplete.is_empty() {
            line.push(text1!(bright_black "{}"(self.autocomplete)));
        }
        if self.cursor < self.cur_line().len() {
            line.push(text1!("{}"(&self.cur_line()[self.cursor..])));
        }

        // insert the cursor
        let ch = line.get_mut(3).map(|t| t.text.remove(0)).unwrap_or(' ');
        let fmt = line
            .get(3)
            .map(|t| t.get_fmt().clone())
            .unwrap_or(Format::default());
        line[2] = Text::of(ch.into()).fmt(fmt).underline();

        // trim the leftmost bits off the left side off
        line[1]
            .text
            .replace_range(..(self.cursor - len_left), if cut_left { "…" } else { "" });
        // rim the rightmost bits of the right side off (-1 to account for cursor)
        let mut trim = len_right - 1;
        let mut trimmed = false;
        for chunk in &mut line[3..] {
            if trim >= chunk.text.len() {
                trim -= chunk.text.len();
            } else if !trimmed {
                chunk
                    .text
                    .replace_range(trim.., if cut_right { "…" } else { "" });
                trimmed = true;
            } else {
                chunk.text.clear();
            }
        }

        screen[0]
            .iter_mut()
            .zip(
                line.iter()
                    .flat_map(|t| t.text.chars().map(|c| Cell::of(c).fmt_of(t)))
                    .chain(iter::repeat(Cell::BLANK)),
            )
            .for_each(|(cell, char)| *cell = char);
    }
}

/// The result of parsing a frame of input.
#[derive(Debug, PartialEq, Eq)]
pub enum TextInputResult<'ti> {
    /// The user didn't do anything that you need to handle.
    ///
    /// For example, an input that this element ignores, or just typing a letter.
    Nothing,
    /// The user pressed Tab to request autocompletion, with the given text.
    ///
    /// `line` is everything up to their current cursor location. `res` is where you put the text you want to show.
    Autocomplete {
        text: &'ti str,
        res: &'ti mut String,
    },
    /// The user pressed Enter to submit a line of text.
    ///
    /// Set `save` to decide whether to store the line in history.
    Submit(String),
}

impl<'s, 'ti> RawAttachment<'s> for &'ti mut TextInput {
    type Output = TextInputResult<'ti>;
    fn raw_attach(self, input: Action, screen: ScreenView<'s>) -> Self::Output {
        // handle input and update state accordingly
        let res = self.input(input);

        // and now render!
        self.render(screen);

        // avoid multiple mutable references (there's a better way, I'm sure, but I don't know it oops)
        res.unwrap_or_else(|| TextInputResult::Autocomplete {
            // hidden potential bug here: to satisfy the borrow checker, we need to directly borrow the field rather
            // than using self.cur_line() which borrows the entire struct (and blocks the &mut)
            // but because this lambda is only called on autocomplete, which selects the line, this works just fine.
            text: &self.line[..self.cursor],
            res: &mut self.autocomplete,
        })
    }
}

#[cfg(test)]
mod test {
    use crate::{
        attachments::test_utils::{
            assert_area_fmt, charat, make_region, make_screen, screen_assert,
        },
        Region,
    };
    use tuig_iosys::{Screen, XY};

    use super::*;

    macro_rules! feed {
        (
            $s:ident, $ti:ident, event $ev:expr
            $( => $( $res:tt )* )?
        ) => {{
            make_region!($s => r(0, 0, *, *, $ev));
            let res = r.attach(&mut $ti);
            $(
                assert_eq!(res, TextInputResult::$($res)*);
            )?
            res
        }};
        ($s:ident, $ti:ident, key $k:expr $( => $( $res:tt )* )? ) => {
            feed!($s, $ti, event Action::KeyPress { key: $k } $( => $( $res )* )?);
            feed!($s, $ti, event Action::KeyRelease { key: $k } => Nothing);
        };
        ($s:ident, $ti:ident, chars $l:expr) => {
            for ch in $l.chars() {
                if ch == '\n' {
                    feed!($s, $ti, key Key::Enter);
                } else {
                    feed!($s, $ti, key Key::Char(ch));
                }
            }
        }
    }

    #[test]
    fn empty_renders_nothing() {
        make_screen!(s(15, 1), r(0, 0, *, *));
        let mut ti = TextInput::new("", 0);
        r.attach(&mut ti);
        screen_assert!(s: fmt 0, 0, " " underline, fmt 1, 0, "              ");
    }

    #[test]
    fn blank_renders_prompt() {
        make_screen!(s(15, 1), r(0, 0, *, *));
        let mut ti = TextInput::new("> ", 0);
        r.attach(&mut ti);
        screen_assert!(s: fmt 0, 0, "> ", fmt 2, 0, " " underline, fmt 3, 0, "            ");
    }

    #[test]
    fn text_rendered_on_keypress() {
        make_screen!(s(15, 1), r(0, 0, *, *, Action::KeyPress { key: Key::Char('z') }));
        let mut ti = TextInput::new("> ", 0);
        r.attach(&mut ti);
        screen_assert!(s: fmt 0, 0, "> z", fmt 3, 0, " " underline, fmt 4, 0, "           ");
        assert_eq!(&ti.line, "z");
    }

    #[test]
    fn text_typed_is_rendered() {
        make_screen!(s(15, 1));
        let mut ti = TextInput::new("> ", 0);
        feed!(s, ti, key Key::Char('a'));
        feed!(s, ti, key Key::Char('b'));
        feed!(s, ti, key Key::Char('c'));
        feed!(s, ti, key Key::Char('d'));
        screen_assert!(s: fmt 0, 0, "> abcd", fmt 6, 0, " " underline, fmt 7, 0, "        ");
    }

    #[test]
    fn overflow_with_cursor_at_end_shows_last() {
        make_screen!(s(15, 1));
        let mut ti = TextInput::new("> ", 0);
        for ch in "0123456789abcdefghijklmnopqrst".chars() {
            feed!(s, ti, key Key::Char(ch));
        }
        screen_assert!(s: fmt 0, 0, "> …jklmnopqrst", fmt 14, 0, " " underline);
    }

    #[test]
    fn overflow_with_cursor_just_before_end_shows_last() {
        make_screen!(s(15, 1));
        let mut ti = TextInput::new("> ", 0);
        for ch in "0123456789abcdefghijklmnopqrst".chars() {
            feed!(s, ti, key Key::Char(ch));
        }
        feed!(s, ti, key Key::Left);
        screen_assert!(s: fmt 0, 0, "> …ijklmnopqrs", fmt 14, 0, "t" underline);
    }

    #[test]
    fn overflow_with_cursor_near_end_shows_last() {
        make_screen!(s(15, 1));
        let mut ti = TextInput::new("> ", 0);
        for ch in "0123456789abcdefghijklmnopqrst".chars() {
            feed!(s, ti, key Key::Char(ch));
        }
        for _ in 0..3 {
            feed!(s, ti, key Key::Left);
        }
        screen_assert!(s: fmt 0, 0, "> …ijklmnopq", fmt 12, 0, "r" underline, fmt 13, 0, "st");
    }

    #[test]
    fn overflow_with_cursor_in_middle_shows_middle() {
        make_screen!(s(15, 1));
        let mut ti = TextInput::new("> ", 0);
        for ch in "0123456789abcdefghijklmnopqrst".chars() {
            feed!(s, ti, key Key::Char(ch));
        }
        for _ in 0..15 {
            feed!(s, ti, key Key::Left);
        }
        screen_assert!(s: fmt 0, 0, "> …e", fmt 4, 0, "f" underline, fmt 5, 0, "ghijklmno…");
    }

    #[test]
    fn overflow_with_cursor_near_beginning_shows_beginning() {
        make_screen!(s(15, 1));
        let mut ti = TextInput::new("> ", 0);
        for ch in "0123456789abcdefghijklmnopqrst".chars() {
            feed!(s, ti, key Key::Char(ch));
        }
        for _ in 0..28 {
            feed!(s, ti, key Key::Left);
        }
        screen_assert!(s: fmt 0, 0, "> 01", fmt 4, 0, "2" underline, fmt 5, 0, "3456789ab…");
    }

    #[test]
    fn left_arrow_moves_cursor() {
        make_screen!(s(15, 1));
        let mut ti = TextInput::new("> ", 0);
        for ch in "123456789".chars() {
            feed!(s, ti, key Key::Char(ch));
        }
        // make sure the screen looks right to begin with
        screen_assert!(s: fmt 0, 0, "> 123456789", fmt 11, 0, " " underline);
        // then move the cursor left
        feed!(s, ti, key Key::Left);
        screen_assert!(s: fmt 0, 0, "> 12345678", fmt 10, 0, "9" underline);
        // and again a couple more times
        feed!(s, ti, key Key::Left);
        screen_assert!(s: fmt 0, 0, "> 1234567", fmt 9, 0, "8" underline, fmt 10, 0, "9");
        feed!(s, ti, key Key::Left);
        screen_assert!(s: fmt 0, 0, "> 123456", fmt 8, 0, "7" underline, fmt 9, 0, "89");
        // then try typing at the cursor
        for ch in "abc".chars() {
            feed!(s, ti, key Key::Char(ch));
        }
        // and make sure it still looks right
        screen_assert!(s: fmt 0, 0, "> 123456abc", fmt 11, 0, "7" underline, fmt 12, 0, "89");
    }

    #[test]
    fn right_arrow_moves_cursor() {
        make_screen!(s(15, 1));
        let mut ti = TextInput::new("> ", 0);
        for ch in "123456789".chars() {
            feed!(s, ti, key Key::Char(ch));
        }
        // make sure the screen looks right to begin with
        screen_assert!(s: fmt 0, 0, "> 123456789", fmt 11, 0, " " underline);
        // then move the cursor left and confirm
        feed!(s, ti, key Key::Left);
        feed!(s, ti, key Key::Left);
        feed!(s, ti, key Key::Left);
        feed!(s, ti, key Key::Left);
        feed!(s, ti, key Key::Left);
        screen_assert!(s: fmt 0, 0, "> 1234", fmt 6, 0, "5" underline, fmt 7, 0, "6789");
        // then move it back right, confirming along the way
        feed!(s, ti, key Key::Right);
        screen_assert!(s: fmt 0, 0, "> 12345", fmt 7, 0, "6" underline, fmt 8, 0, "789");
        feed!(s, ti, key Key::Right);
        screen_assert!(s: fmt 0, 0, "> 123456", fmt 8, 0, "7" underline, fmt 9, 0, "89");
        feed!(s, ti, key Key::Right);
        screen_assert!(s: fmt 0, 0, "> 1234567", fmt 9, 0, "8" underline, fmt 10, 0, "9");
        // then try typing at the cursor
        for ch in "abc".chars() {
            feed!(s, ti, key Key::Char(ch));
        }
        // and make sure it still looks right
        screen_assert!(s: fmt 0, 0, "> 1234567abc", fmt 12, 0, "8" underline, fmt 13, 0, "9");
    }

    #[test]
    fn home_moves_cursor() {
        make_screen!(s(15, 1));
        let mut ti = TextInput::new("> ", 0);
        for ch in "123456789".chars() {
            feed!(s, ti, key Key::Char(ch));
        }
        // make sure the screen looks right to begin with
        screen_assert!(s: fmt 0, 0, "> 123456789", fmt 11, 0, " " underline);
        // then hit the home button
        feed!(s, ti, key Key::Home);
        screen_assert!(s: fmt 0, 0, "> ", fmt 2, 0, "1" underline, fmt 3, 0, "23456789");
        // and again a couple more times, make sure it doesn't change
        feed!(s, ti, key Key::Home);
        screen_assert!(s: fmt 0, 0, "> ", fmt 2, 0, "1" underline, fmt 3, 0, "23456789");
        feed!(s, ti, key Key::Home);
        screen_assert!(s: fmt 0, 0, "> ", fmt 2, 0, "1" underline, fmt 3, 0, "23456789");
        // then try typing at the cursor
        for ch in "abc".chars() {
            feed!(s, ti, key Key::Char(ch));
        }
        // and make sure it still looks right
        screen_assert!(s: fmt 0, 0, "> abc", fmt 5, 0, "1" underline, fmt 6, 0, "23456789");
    }

    #[test]
    fn end_moves_cursor() {
        make_screen!(s(15, 1));
        let mut ti = TextInput::new("> ", 0);
        for ch in "123456789".chars() {
            feed!(s, ti, key Key::Char(ch));
        }
        // make sure the screen looks right to begin with
        screen_assert!(s: fmt 0, 0, "> 123456789", fmt 11, 0, " " underline);
        // then move the cursor left and confirm
        feed!(s, ti, key Key::Left);
        feed!(s, ti, key Key::Left);
        feed!(s, ti, key Key::Left);
        feed!(s, ti, key Key::Left);
        feed!(s, ti, key Key::Left);
        screen_assert!(s: fmt 0, 0, "> 1234", fmt 6, 0, "5" underline, fmt 7, 0, "6789");
        // then move it back to the end
        feed!(s, ti, key Key::End);
        screen_assert!(s: fmt 0, 0, "> 123456789", fmt 11, 0, " " underline);
        // then try typing at the cursor
        feed!(s, ti, chars "abc");
        // and make sure it still looks right
        screen_assert!(s: fmt 0, 0, "> 123456789abc", fmt 14, 0, " " underline);
    }

    #[test]
    fn overflow_with_cursor_at_beginning_shows_beginning() {
        make_screen!(s(15, 1));
        let mut ti = TextInput::new("> ", 0);
        for ch in "0123456789abcdefghijklmnopqrst".chars() {
            feed!(s, ti, key Key::Char(ch));
        }
        for _ in 0..30 {
            feed!(s, ti, key Key::Left);
        }
        screen_assert!(s: fmt 0, 0, "> ", fmt 2, 0, "0" underline, fmt 3, 0, "123456789ab…");
    }

    #[test]
    fn autocomplete_clipped_shows_gray_dots() {
        make_screen!(s(15, 1));
        let mut ti = TextInput::new("> ", 0);
        const TEXT: &str = "0123456789abcdefghijklmnopqrst";
        feed!(s, ti, chars TEXT);
        for _ in 0..20 {
            feed!(s, ti, key Key::Left);
        }
        // first: make sure we know what it should look like
        screen_assert!(s: fmt 0, 0, "> …9", fmt 4, 0, "a" underline, fmt 5, 0, "bcdefghij…");
        // autocomplete: should insert `ABCDEFGHIJ_____`, which gets cut off
        match feed!(s, ti, event Action::KeyPress { key: Key::Tab }) {
            TextInputResult::Autocomplete { res, .. } => *res = "ABCDEFGHIJ_____".into(),
            _ => panic!("tab did not trigger TextInputResult::Autocomplete"),
        }
        feed!(s, ti, event Action::Redraw);
        screen_assert!(s:
            fmt 0, 0, "> …9", fmt 4, 0,
            "A" underline bright_black, fmt 5, 0, "BCDEFGHIJ…" bright_black,
        );
    }

    #[test]
    fn autocomplete_barely_unclipped_shows_white_dots() {
        make_screen!(s(15, 1));
        let mut ti = TextInput::new("> ", 0);
        const TEXT: &str = "0123456789abcdefghijklmnopqrst";
        feed!(s, ti, chars TEXT);
        for _ in 0..20 {
            feed!(s, ti, key Key::Left);
        }
        // first: make sure we know what it should look like
        screen_assert!(s: fmt 0, 0, "> …9", fmt 4, 0, "a" underline, fmt 5, 0, "bcdefghij…");
        // autocomplete: should insert `ABCDEFGHIJ`, which just barely fits, with normal text cut off after
        match feed!(s, ti, event Action::KeyPress { key: Key::Tab }) {
            TextInputResult::Autocomplete { res, .. } => *res = "ABCDEFGHIJ".into(),
            _ => panic!("tab did not trigger TextInputResult::Autocomplete"),
        }
        feed!(s, ti, event Action::Redraw);
        screen_assert!(s:
            fmt 0, 0, "> …9", fmt 4, 0,
            "A" underline bright_black, fmt 5, 0, "BCDEFGHIJ" bright_black,
            fmt 14, 0, "…",
        );
    }

    #[test]
    fn press_enter_returns_input_text() {
        make_screen!(s(15, 1));
        let mut ti = TextInput::new("> ", 0);
        const TEXT: &str = "0123456789abcdefghijklmnopqrst";
        feed!(s, ti, chars TEXT);
        feed!(s, ti, event Action::KeyPress { key: Key::Enter } => Submit(TEXT.into()));
        screen_assert!(s: fmt 0, 0, "> ", fmt 2, 0, " " underline, fmt 3, 0, "            ");
    }

    #[test]
    fn press_enter_returns_input_text_when_cursor_not_at_end() {
        make_screen!(s(15, 1));
        let mut ti = TextInput::new("> ", 0);
        const TEXT: &str = "0123456789abcdefghijklmnopqrst";
        feed!(s, ti, chars TEXT);
        for _ in 0..5 {
            feed!(s, ti, key Key::Left);
        }
        feed!(s, ti, event Action::KeyPress { key: Key::Enter } => Submit(TEXT.into()));
        screen_assert!(s: fmt 0, 0, "> ", fmt 2, 0, " " underline, fmt 3, 0, "            ");
    }

    #[test]
    fn press_tab_triggers_autocomplete() {
        make_screen!(s(15, 1));
        let mut ti = TextInput::new("> ", 0);
        feed!(s, ti, chars "abcdefg");
        feed!(s, ti, key Key::Left);
        feed!(s, ti, key Key::Left);
        feed!(s, ti, key Key::Left);
        // line should now be abcd_fg
        match feed!(s, ti, event Action::KeyPress { key: Key::Tab }) {
            TextInputResult::Autocomplete { text, res } => {
                assert_eq!(text, "abcd");
                assert_eq!(res, "");
            }
            _ => panic!("tab did not trigger TextInputResult::Autocomplete"),
        }
    }

    #[test]
    fn autocomplete_render_correct() {
        make_screen!(s(15, 1));
        let mut ti = TextInput::new("> ", 0);
        feed!(s, ti, chars "abcdefg");
        feed!(s, ti, key Key::Left);
        feed!(s, ti, key Key::Left);
        feed!(s, ti, key Key::Left);
        // line should now be abcd_fg
        match feed!(s, ti, event Action::KeyPress { key: Key::Tab }) {
            TextInputResult::Autocomplete { res, .. } => *res = "mlem".into(),
            _ => panic!("tab did not trigger TextInputResult::Autocomplete"),
        }
        // immediately after hitting tab, it shouldn't show
        screen_assert!(s: fmt 0, 0, "> abcd", fmt 6, 0, "e" underline, fmt 7, 0, "fg      ");
        // then we redraw and it should show up
        feed!(s, ti, event Action::Redraw => Nothing);
        screen_assert!(s:
            fmt 0, 0, "> abcd",
            fmt 6, 0, "m" bright_black underline, fmt 7, 0, "lem" bright_black,
            fmt 10, 0, "efg  "
        );
    }

    #[test]
    fn autocomplete_goes_away_after_keypress() {
        make_screen!(s(15, 1));
        let mut ti = TextInput::new("> ", 0);
        feed!(s, ti, chars "abcdefg");
        feed!(s, ti, key Key::Left);
        feed!(s, ti, key Key::Left);
        feed!(s, ti, key Key::Left);
        match feed!(s, ti, event Action::KeyPress { key: Key::Tab }) {
            TextInputResult::Autocomplete { res, .. } => *res = "mlem".into(),
            _ => panic!("tab did not trigger TextInputResult::Autocomplete"),
        }
        feed!(s, ti, event Action::Redraw);
        // ensure the autocomplete was drawn
        screen_assert!(s:
            fmt 0, 0, "> abcd",
            fmt 6, 0, "m" bright_black underline, fmt 7, 0, "lem" bright_black,
            fmt 10, 0, "efg  "
        );
        // mouse movement shouldn't get rid of it
        feed!(s, ti, event Action::MouseMove { pos: XY(0, 0) });
        screen_assert!(s:
            fmt 0, 0, "> abcd",
            fmt 6, 0, "m" bright_black underline, fmt 7, 0, "lem" bright_black,
            fmt 10, 0, "efg  "
        );
        // type a char to watch the autocomplete go away
        feed!(s, ti, event Action::KeyPress { key: Key::Char('z') });
        screen_assert!(s:
            fmt 0, 0, "> abcdz", fmt 7, 0, "e" underline, fmt 8, 0, "fg     "
        );
    }

    #[test]
    fn store_alone_adds_to_history() {
        make_screen!(s(15, 1));
        let mut ti = TextInput::new("> ", 2);
        ti.store("abcdef".into());
        feed!(s, ti, event Action::Redraw);
        // ensure it's blank
        screen_assert!(s: fmt 0, 0, "> ", fmt 2, 0, " " underline, fmt 3, 0, "            ");
        // then hit up and ensure the previous line is there
        feed!(s, ti, key Key::Up);
        screen_assert!(s: fmt 0, 0, "> abcdef", fmt 8, 0, " " underline, fmt 9, 0, "      ");
    }

    #[test]
    fn submit_alone_doesnt_add_to_history() {
        make_screen!(s(15, 1));
        let mut ti = TextInput::new("> ", 2);
        feed!(s, ti, chars "abcdef\n");
        // ensure it's blank
        screen_assert!(s: fmt 0, 0, "> ", fmt 2, 0, " " underline, fmt 3, 0, "            ");
        // then hit up and ensure it's still blank
        feed!(s, ti, key Key::Up);
        screen_assert!(s: fmt 0, 0, "> ", fmt 2, 0, " " underline, fmt 3, 0, "            ");
    }

    #[test]
    fn down_restores_current_line() {
        make_screen!(s(15, 1));
        let mut ti = TextInput::new("> ", 2);
        ti.store("abcdef".into());
        // new text on the current line, without submitting
        feed!(s, ti, chars "01234");
        screen_assert!(s: fmt 0, 0, "> 01234", fmt 7, 0, " " underline);
        // go to the previous line, ensure that's correct
        feed!(s, ti, key Key::Up);
        screen_assert!(s: fmt 0, 0, "> abcdef", fmt 8, 0, " " underline);
        // go back to the current line, and ensure that's right
        feed!(s, ti, key Key::Down);
        screen_assert!(s: fmt 0, 0, "> 01234", fmt 7, 0, " " underline);
    }

    #[test]
    fn cursor_move_doesnt_select_history() {
        make_screen!(s(15, 1));
        let mut ti = TextInput::new("> ", 2);
        ti.store("abcdef".into());
        feed!(s, ti, chars "01234");
        feed!(s, ti, key Key::Up);
        // move a bit and make sure the cursor moved
        screen_assert!(s: fmt 0, 0, "> abcdef", fmt 8, 0, " " underline);
        feed!(s, ti, key Key::Left);
        feed!(s, ti, key Key::Left);
        feed!(s, ti, key Key::Left);
        screen_assert!(s: fmt 0, 0, "> abc", fmt 5, 0, "d" underline, fmt 6, 0, "ef");
        // go back to the current line, whose content should be unchanged (and the cursor should be at the end)
        feed!(s, ti, key Key::Down);
        screen_assert!(s: fmt 0, 0, "> 01234", fmt 7, 0, " " underline);
    }

    #[test]
    fn typing_selects_history() {
        make_screen!(s(15, 1));
        let mut ti = TextInput::new("> ", 2);
        ti.store("abcdef".into());
        feed!(s, ti, chars "01234");
        feed!(s, ti, key Key::Up);
        // type a character
        feed!(s, ti, key Key::Char('z'));
        screen_assert!(s: fmt 0, 0, "> abcdefz", fmt 9, 0, " " underline);
        // press up to load the previous history and ensure it's there
        feed!(s, ti, key Key::Up);
        screen_assert!(s: fmt 0, 0, "> abcdef", fmt 8, 0, " " underline);
    }

    #[test]
    fn enter_selects_history_and_submits() {
        make_screen!(s(15, 1));
        let mut ti = TextInput::new("> ", 2);
        // prep history
        ti.store("abc".into());
        ti.store("1234".into());
        // up arrow should give us `01234`, then `abc`
        feed!(s, ti, key Key::Up);
        feed!(s, ti, key Key::Up);
        // submit, and ensure we get the relevant text
        feed!(s, ti, event Action::KeyPress { key: Key::Enter } => Submit("abc".into()));
        feed!(s, ti, event Action::KeyRelease { key: Key::Enter });
        // ensure the screen is as it should be
        screen_assert!(s: fmt 0, 0, "> ", fmt 2, 0, " " underline);
        // up should show us "1234" because we didn't store "abc" but we did move down
        feed!(s, ti, key Key::Up);
        screen_assert!(s: fmt 0, 0, "> 1234", fmt 6, 0, " " underline);
        // up again should show us "abc" (the original)
        feed!(s, ti, key Key::Up);
        screen_assert!(s: fmt 0, 0, "> abc", fmt 5, 0, " " underline);
    }
}
