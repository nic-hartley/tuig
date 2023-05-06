use alloc::{collections::VecDeque, string::String};

use crate::{
    fmt::{Cell, FormattedExt},
    text,
    ui::ScreenView,
    Action,
};

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
/// Accordingly, [`Attachment`] is implemented for `&mut TextInput`, not `TextInput` itself, so you'll use it like:
///
/// ```no_run
/// # use tuig_iosys::ui::{Region, elements::{TextInput, TextInputResult}};
/// let region = //...
/// # Region::empty();
/// let text_input = // ...
/// # TextInput::new();
/// match region.attach(&mut text_input) {
///   TextInputResult::Nothing => (),
///   TextInputResult::Autocomplete { text, res } => *res = text.into(),
///   TextInputResult::Submit(s) => println!("enter pressed: {}", s),
/// }
/// ```
///
/// [`Region::attach`]ing this will return a [`TextInputResult`], which is how you'll interact with autocomplete. To
/// use the history features, see [`TextInput::store`].
pub struct TextInput {
    /// A bit of fixed, uneditable text at the beginning of the text input, to signal the user to type.
    prompt: String,

    /// The current line of text being edited
    line: String,
    /// Which character index the cursor is just before (so `cursor == line.len()` means the cursor is at the end)
    cursor: usize,

    /// The caller-specified autocomplete text
    autocomplete: String,

    /// Previous lines we were told to save
    ///
    /// The history goes older to newer from front to back, so new lines are added with `push_back` and old ones are
    /// removed with `pop_front`.
    history: VecDeque<String>,
    /// Current position, if scrolling back through history, or `history.len()`
    histpos: usize,
    /// Maximum number of history elements
    histcap: usize,
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
        }
        self.history.push_back(line);
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
    fn raw_attach(self, input: Action, mut screen: ScreenView<'s>) -> Self::Output {
        let res = match input {
            _ => TextInputResult::Nothing,
        };

        let text = text![];
        // TODO: generate the base text
        // TODO: slice according to cursor position
        // TODO: underline cursor character

        screen[0]
            .iter_mut()
            .zip(
                text.iter()
                    .flat_map(|t| t.text.chars().map(|c| Cell::of(c).fmt_of(t))),
            )
            .for_each(|(cell, char)| *cell = char);

        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{XY, fmt::Cell, ui::{Region, elements::test_utils::*}, fmt::FormattedExt, Screen, Key};

    macro_rules! feed {
        ($s:ident, $ti:ident, event $ev:expr) => {{
            make_region!($s => r(0, 0, *, *, $ev));
            r.attach(&mut $ti)
        }};
        ($s:ident, $ti:ident, key $k:expr) => {
            assert_eq!(feed!($s, $ti, event Action::KeyPress { key: $k }), TextInputResult::Nothing);
            assert_eq!(feed!($s, $ti, event Action::KeyRelease { key: $k }), TextInputResult::Nothing);
        };
    }

    #[test]
    fn empty_renders_nothing() {
        make_screen!(s(15, 1), r(0, 0, *, *));
        let mut ti = TextInput::new("", 0);
        r.attach(&mut ti);
        screen_assert!(s: blank .., ..);
    }

    #[test]
    fn blank_renders_prompt() {
        make_screen!(s(15, 1), r(0, 0, *, *));
        let mut ti = TextInput::new("> ", 0);
        r.attach(&mut ti);
        screen_assert!(s: fmt 0, 0, "> ", blank 2.., ..);
    }

    #[test]
    fn text_rendered_on_keypress() {
        make_screen!(s(15, 1), r(0, 0, *, *, Action::KeyPress { key: Key::Char('z') }));
        let mut ti = TextInput::new("> ", 0);
        r.attach(&mut ti);
        screen_assert!(s: fmt 0, 0, "> z", blank 2.., ..);
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
        screen_assert!(s: fmt 0, 0, "> abcd", blank 6.., ..);
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
    fn overflow_with_cursor_at_beginning_shows_beginning() {
        make_screen!(s(15, 1));
        let mut ti = TextInput::new("> ", 0);
        for ch in "0123456789abcdefghijklmnopqrst".chars() {
            feed!(s, ti, key Key::Char(ch));
        }
        for _ in 0..28 {
            feed!(s, ti, key Key::Left);
        }
        screen_assert!(s: fmt 0, 0, "> ", fmt 2, 0, "0" underline, fmt 3, 0, "123456789ab…");
    }

    #[test]
    fn press_enter_returns_input_text() {
        make_screen!(s(15, 1));
        let mut ti = TextInput::new("> ", 0);
        const TEXT: &str = "0123456789abcdefghijklmnopqrst";
        for ch in TEXT.chars() {
            feed!(s, ti, key Key::Char(ch));
        }
        assert_eq!(feed!(s, ti, event Action::KeyPress { key: Key::Enter }), TextInputResult::Nothing);
        assert_eq!(feed!(s, ti, event Action::KeyRelease { key: Key::Enter }), TextInputResult::Submit(TEXT.into()));
        screen_assert!(s: fmt 0, 0, "> ", blank 2.., ..);
    }

    // triggers autocomplete when you press tab
    // render returning autocomplete is just text, next render includes the autocomplete
    // autocomplete goes away after keypress
}
