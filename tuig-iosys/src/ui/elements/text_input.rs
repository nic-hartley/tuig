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
    // empty renders nothing
    // renders text on the same frame as the keypress
    // with some text, renders that text
    // with too much text, cursor at end, renders the last few
    // with too much text, cursor *near* end, renders the same
    // with too much text, cursor away from end, renders the middle few
    // with too much text, cursor near beginning, renders almost the first few
    // with too much text, cursor beginning, renders the first few
    // returns the input text and clears when you press enter
    // triggers autocomplete when you press tab
    // render returning autocomplete is just text, next render includes the autocomplete
    // autocomplete goes away after keypress
}
