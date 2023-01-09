use std::mem;

use crate::text;

use super::{input::{Action, Key}, clifmt::Text};

/// Indicates what the text input needs from its owner
pub enum TextInputRequest {
    /// Action doesn't require any response.
    Nothing,
    /// Autocomplete has been requested, with the given text.
    Autocomplete,
    /// User has submitted a line by pressing Enter.
    Line(String),
}

/// Allows the user to input text, rendering it to a bounded area and offering hooks for tab-based autocomplete.
#[derive(Clone, Default)]
pub struct TextInput {
    /// prompt displayed before the user's text
    prompt: String,
    /// line(s) currently being typed
    line: String,
    /// cursor position in the line being typed
    cursor: usize,
    /// any autocomplete that's been requested
    autocomplete: String,
    /// whether the textbox needs to be redrawn since it was last rendered
    tainted: bool,
}

impl TextInput {
    /// Create a new text input element.
    pub fn new(prompt: &str) -> Self {
        Self {
            prompt: prompt.into(),
            line: String::new(),
            cursor: 0,
            autocomplete: String::new(),
            tainted: true,
        }
    }

    pub fn completable(&self) -> &str {
        &self.line[..self.cursor]
    }

    pub fn set_complete(&mut self, text: String) {
        self.autocomplete = text;
        self.tainted = true;
    }

    /// Whether this text box needs to be redrawn.
    pub fn tainted(&self) -> bool {
        self.tainted
    }

    fn keypress(&mut self, key: Key) -> TextInputRequest {
        match key {
            Key::Char(ch) => {
                self.line.insert(self.cursor, ch);
                self.cursor += 1;
            }
            Key::Backspace if self.cursor > 0 => {
                self.line.remove(self.cursor - 1);
                self.cursor -= 1;
            }
            Key::Delete if self.cursor < self.line.len() => {
                self.line.remove(self.cursor);
            }
            Key::Left if self.cursor > 0 => self.cursor -= 1,
            Key::Right if self.cursor < self.line.len() => self.cursor += 1,
            // TODO: up/down to scroll through history
            Key::Tab => {
                if self.autocomplete.is_empty() {
                    return TextInputRequest::Autocomplete;
                } else {
                    self.line.insert_str(self.cursor, &self.autocomplete);
                    self.cursor += self.autocomplete.len();
                }
            }
            Key::Enter => {
                self.cursor = 0;
                self.autocomplete = String::new();
                let old_line = mem::replace(&mut self.line, String::new());
                return TextInputRequest::Line(old_line);
            }
            // return early to skip tainting / clearing autocomplete
            _ => return TextInputRequest::Nothing,
        }
        self.autocomplete = String::new();
        self.tainted = true;
        TextInputRequest::Nothing
    }

    /// Handles an [`Action`] which should go to the textbox, for things like typing and autocompletion.
    /// 
    /// The type this returns indicates what needs to be done 
    pub fn action(&mut self, action: Action) -> TextInputRequest {
        match action {
            Action::KeyPress { key } => self.keypress(key),
            _ => TextInputRequest::Nothing,
        }
    }

    /// Builds a `Vec<Text>` with the prompt line, for rendering.
    pub fn render<'s>(&mut self) -> Vec<Text> {
        self.tainted = false;
        if self.cursor == self.line.len() {
            if self.autocomplete.is_empty() {
                text![
                    "{}"(self.prompt),
                    bright_white "{}"(self.line),
                    bright_white underline " ",
                ]
            } else {
                text![
                    "{}"(self.prompt),
                    bright_white "{}"(self.line),
                    bright_black underline "{}"(&self.autocomplete[..1]),
                    bright_black "{}"(&self.autocomplete[1..]),
                ]
            }
        } else {
            if self.autocomplete.is_empty() {
                text![
                    "{}"(self.prompt),
                    bright_white "{}"(&self.line[..self.cursor]),
                    bright_white underline "{}"(&self.line[self.cursor..self.cursor+1]),
                    bright_white "{}"(&self.line[self.cursor+1..]),
                ]
            } else {
                text![
                    "{}"(self.prompt),
                    bright_white "{}"(&self.line[..self.cursor]),
                    bright_black underline "{}"(&self.autocomplete[..1]),
                    bright_black "{}"(&self.autocomplete[1..]),
                    bright_white "{}"(&self.line[self.cursor..]),
                ]
            }
        }
    }
}
