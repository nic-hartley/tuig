use alloc::{collections::VecDeque, string::String};

use crate::{ui::ScreenView, Action, text, fmt::{Cell, FormattedExt}, text1};

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

        screen[0].iter_mut().zip(
            text.iter().flat_map(|t| t.text.chars().map(|c| Cell::of(c).fmt_of(t)))
        ).for_each(|(cell, char)| *cell = char);

        res
    }
}

#[cfg(testa)]
mod test {
    use super::*;

    /// Feed a series of inputs to a TextInput, asserting that it returns `TextInputRequest::Nothing`, and maybe
    /// whether it's tainted afterwards.
    macro_rules! feed {
        ( $ti:ident: $(
            $key:ident $( ( $content:expr ) )? $( $side:ident )?
        ),* ) => {
            $(
                assert!(matches!(
                    feed!(@@one $ti: $key $( ($content) )?),
                    TextInputRequest::Nothing | TextInputRequest::Redraw,
                ));
            )*
        };
        ( @@one $ti:ident: String($val:literal) ) => {
            {
                let mut res = None;
                let mut cap = false;
                for ch in $val.chars() {
                    let new_cap = ch.is_uppercase();
                    if new_cap != cap {
                        if new_cap {
                            assert_eq!(feed!(@@one $ti: LeftShift KeyPress), TextInputRequest::Nothing);
                        } else {
                            assert_eq!(feed!(@@one $ti: LeftShift KeyRelease), TextInputRequest::Nothing);
                        }
                        cap = new_cap;
                    }
                    let cur = feed!(@@one $ti: Char(ch));
                    if let Some(prev) = &res {
                        assert_eq!(&cur, prev, "different results across fed String");
                    } else {
                        res = Some(cur);
                    }
                }
                if cap {
                    assert_eq!(feed!(@@one $ti: LeftShift KeyRelease), TextInputRequest::Nothing);
                }
                res.expect("test is broken: tried to feed empty string")
            }
        };
        ( @@one $ti:ident: String($_:tt) $side:ident ) => {
            compile_error!("Cannot KeyPress/KeyRelease a String");
        };
        ( @@one $ti:ident: $key:ident $( ($content:expr) )? ) => {
            {
                let res1 = feed!(@@one $ti: $key $( ( $content ) )? KeyPress );
                let res2 = feed!(@@one $ti: $key $( ( $content ) )? KeyRelease );
                assert!(res1.test_eq(&res2), "press/release fed char differed");
                res1
            }
        };
        ( @@one $ti:ident: $key:ident $( ($content:expr) )? $side:ident ) => {
            $ti.action(Action::$side { key: Key::$key $( ($content) )? })
        };
    }

    #[test]
    fn blank_renders_to_prompt() {
        let ti = TextInput::new("> ", 0);
        assert_eq!(ti.render(), text!["> ", bright_white "", underline " "]);
    }

    #[test]
    fn text_renders_to_prompt() {
        let mut ti = TextInput::new("> ", 0);
        feed!(ti: String("abcdef"));
        assert_eq!(
            ti.render(),
            text!["> ", bright_white "abcdef", underline " "]
        );
    }

    #[test]
    fn blank_renders_to_prompt_with_autocomplete() {
        let mut ti = TextInput::new("> ", 0);
        ti.set_complete("mlem".into());
        assert_eq!(
            ti.render(),
            text!["> ", bright_white "", underline bright_black "m", bright_black "lem"]
        );
    }

    #[test]
    fn text_renders_to_prompt_with_autocomplete() {
        let mut ti = TextInput::new("> ", 0);
        feed!(ti: String("abcdef"));
        ti.set_complete("mlem".into());
        assert_eq!(
            ti.render(),
            text!["> ", bright_white "abcdef", bright_black underline "m", bright_black "lem"]
        );
    }

    #[test]
    fn text_renders_to_prompt_moved_cursor() {
        let mut ti = TextInput::new("> ", 0);
        feed!(ti: String("abcdef"), Left, Left);
        assert_eq!(
            ti.render(),
            text!["> ", bright_white "abcd", underline bright_white "e", bright_white "f"]
        );
    }

    #[test]
    fn text_renders_to_prompt_with_autocomplete_moved_cursor() {
        let mut ti = TextInput::new("> ", 0);
        feed!(ti: String("abcdef"), Left, Left, Left, Right);
        ti.set_complete("mlem".into());
        assert_eq!(
            ti.render(),
            text!["> ", bright_white "abcd", underline bright_black "m", bright_black "lem", bright_white "ef"]
        );
    }

    #[test]
    fn typing_uppercase() {
        let mut ti = TextInput::new("> ", 0);
        feed!(ti: String("abCDef"));
        assert_eq!(
            ti.render(),
            text!["> ", bright_white "abCDef", underline " "]
        );
    }

    #[test]
    fn backspacing_chars() {
        let mut ti = TextInput::new("> ", 0);
        feed!(ti: String("abcd"), Backspace, Backspace, String("ef"));
        assert_eq!(ti.render(), text!["> ", bright_white "abef", underline " "]);
    }

    #[test]
    fn deleting_chars() {
        let mut ti = TextInput::new("> ", 0);
        feed!(ti: String("abcd"), Left, Left, Delete, Delete, String("ef"));
        assert_eq!(ti.render(), text!["> ", bright_white "abef", underline " "]);
    }

    #[test]
    fn tab_triggers_autocomplete() {
        let mut ti = TextInput::new("> ", 0);
        feed!(ti: String("abc"));
        assert_eq!(
            ti.action(Action::KeyPress { key: Key::Tab }),
            TextInputRequest::Autocomplete
        );
        assert_eq!(ti.completable(), "abc");
        ti.set_complete("mlem".into());
        assert_eq!(
            ti.render(),
            text!["> ", bright_white "abc", underline bright_black "m", bright_black "lem"]
        );
    }

    #[test]
    fn tab_twice_finishes_autocomplete() {
        let mut ti = TextInput::new("> ", 0);
        feed!(ti: String("abc"));
        assert_eq!(
            ti.action(Action::KeyPress { key: Key::Tab }),
            TextInputRequest::Autocomplete
        );
        assert_eq!(ti.completable(), "abc");
        ti.set_complete("mlem".into());
        feed!(ti: Tab);
        assert_eq!(
            ti.render(),
            text!["> ", bright_white "abcmlem", underline " "]
        );
    }

    #[test]
    fn enter_sends_line() {
        let mut ti = TextInput::new("> ", 0);
        feed!(ti: String("abc"));
        assert_eq!(
            ti.action(Action::KeyPress { key: Key::Enter }),
            TextInputRequest::Line("abc".into())
        );
        assert_eq!(ti.render(), text!["> ", bright_white "", underline " "]);
    }

    #[test]
    fn history_scrolls_with_arrows() {
        let mut ti = TextInput::new("> ", 10);
        feed!(ti: String("abc"));
        assert_eq!(
            ti.action(Action::KeyPress { key: Key::Enter }),
            TextInputRequest::Line("abc".into())
        );
        feed!(ti: String("def"));
        assert_eq!(
            ti.action(Action::KeyPress { key: Key::Enter }),
            TextInputRequest::Line("def".into())
        );
        feed!(ti: String("ghi"));
        assert_eq!(
            ti.action(Action::KeyPress { key: Key::Enter }),
            TextInputRequest::Line("ghi".into())
        );
        assert_eq!(ti.render(), text!["> ", bright_white "", underline " "]);
        feed!(ti: Up, Up, Up);
        assert_eq!(ti.render(), text!["> ", bright_white "abc", underline " "]);
        feed!(ti: Down);
        assert_eq!(ti.render(), text!["> ", bright_white "def", underline " "]);
    }

    #[test]
    fn history_scroll_to_bottom_doesnt_reset_line() {
        let mut ti = TextInput::new("> ", 10);
        feed!(ti: String("abc"));
        assert_eq!(
            ti.action(Action::KeyPress { key: Key::Enter }),
            TextInputRequest::Line("abc".into())
        );
        feed!(ti: String("def"));
        assert_eq!(
            ti.action(Action::KeyPress { key: Key::Enter }),
            TextInputRequest::Line("def".into())
        );
        feed!(ti: String("ghi"));
        assert_eq!(
            ti.action(Action::KeyPress { key: Key::Enter }),
            TextInputRequest::Line("ghi".into())
        );
        feed!(ti: String("jkl"));
        assert_eq!(ti.render(), text!["> ", bright_white "jkl", underline " "]);
        feed!(ti: Up);
        assert_eq!(ti.render(), text!["> ", bright_white "ghi", underline " "]);
        feed!(ti: Down);
        assert_eq!(ti.render(), text!["> ", bright_white "jkl", underline " "]);
    }

    #[test]
    fn history_selects_with_typing() {
        let mut ti = TextInput::new("> ", 10);
        feed!(ti: String("abc"));
        assert_eq!(
            ti.action(Action::KeyPress { key: Key::Enter }),
            TextInputRequest::Line("abc".into())
        );
        feed!(ti: String("def"));
        assert_eq!(
            ti.action(Action::KeyPress { key: Key::Enter }),
            TextInputRequest::Line("def".into())
        );
        feed!(ti: String("ghi"));
        assert_eq!(
            ti.action(Action::KeyPress { key: Key::Enter }),
            TextInputRequest::Line("ghi".into())
        );
        feed!(ti: Up);
        assert_eq!(ti.render(), text!["> ", bright_white "ghi", underline " "]);
        feed!(ti: Char('j'));
        assert_eq!(ti.render(), text!["> ", bright_white "ghij", underline " "]);
        feed!(ti: Up);
        assert_eq!(ti.render(), text!["> ", bright_white "ghi", underline " "]);
        feed!(ti: Up);
        assert_eq!(ti.render(), text!["> ", bright_white "def", underline " "]);
    }

    #[test]
    fn history_selects_with_backspace() {
        let mut ti = TextInput::new("> ", 10);
        feed!(ti: String("abc"));
        assert_eq!(
            ti.action(Action::KeyPress { key: Key::Enter }),
            TextInputRequest::Line("abc".into())
        );
        feed!(ti: String("def"));
        assert_eq!(
            ti.action(Action::KeyPress { key: Key::Enter }),
            TextInputRequest::Line("def".into())
        );
        feed!(ti: String("ghi"));
        assert_eq!(
            ti.action(Action::KeyPress { key: Key::Enter }),
            TextInputRequest::Line("ghi".into())
        );
        feed!(ti: Up);
        assert_eq!(ti.render(), text!["> ", bright_white "ghi", underline " "]);
        feed!(ti: Backspace);
        assert_eq!(ti.render(), text!["> ", bright_white "gh", underline " "]);
        feed!(ti: Up);
        assert_eq!(ti.render(), text!["> ", bright_white "ghi", underline " "]);
        feed!(ti: Up);
        assert_eq!(ti.render(), text!["> ", bright_white "def", underline " "]);
    }

    #[test]
    fn history_selects_with_enter() {
        let mut ti = TextInput::new("> ", 10);
        feed!(ti: String("abc"));
        assert_eq!(
            ti.action(Action::KeyPress { key: Key::Enter }),
            TextInputRequest::Line("abc".into())
        );
        feed!(ti: String("def"));
        assert_eq!(
            ti.action(Action::KeyPress { key: Key::Enter }),
            TextInputRequest::Line("def".into())
        );
        feed!(ti: String("ghi"));
        assert_eq!(
            ti.action(Action::KeyPress { key: Key::Enter }),
            TextInputRequest::Line("ghi".into())
        );
        feed!(ti: Up);
        assert_eq!(ti.render(), text!["> ", bright_white "ghi", underline " "]);
        assert_eq!(
            ti.action(Action::KeyPress { key: Key::Enter }),
            TextInputRequest::Line("ghi".into())
        );
        assert_eq!(ti.render(), text!["> ", bright_white "", underline " "]);
        feed!(ti: Up);
        assert_eq!(ti.render(), text!["> ", bright_white "ghi", underline " "]);
        feed!(ti: Up);
        assert_eq!(ti.render(), text!["> ", bright_white "ghi", underline " "]);
        feed!(ti: Up);
        assert_eq!(ti.render(), text!["> ", bright_white "def", underline " "]);
    }
}
