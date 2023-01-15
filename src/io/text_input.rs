use std::{mem, collections::VecDeque};

use crate::text;

use super::{
    clifmt::Text,
    input::{Action, Key},
};

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

    /// whether the shift key is currently being held
    shift: bool,

    /// previous lines entered, for scrolling through
    history: VecDeque<String>,
    /// where in the history we are. (history.len() = next line.)
    hist_pos: usize,
    /// the number of history items to store
    hist_cap: usize,
}

impl TextInput {
    /// Create a new text input element.
    pub fn new(prompt: &str, history: usize) -> Self {
        Self {
            prompt: prompt.into(),
            line: String::new(),
            cursor: 0,
            autocomplete: String::new(),
            tainted: true,
            shift: false,
            history: Default::default(),
            hist_pos: 0,
            hist_cap: history,
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

    fn cur_line(&self) -> &str {
        if self.hist_pos < self.history.len() {
            &self.history[self.hist_pos]
        } else {
            &self.line
        }
    }

    fn pick_hist(&mut self) {
        if self.hist_pos == self.history.len() {
            return;
        }
        self.line = self.history[self.hist_pos].clone();
        self.hist_pos = self.history.len();
    }

    fn keypress(&mut self, key: Key) -> TextInputRequest {
        match key {
            Key::Char(ch) => {
                self.pick_hist();
                let chs: String = if self.shift {
                    ch.to_uppercase().collect()
                } else {
                    ch.to_lowercase().collect()
                };
                self.line.insert_str(self.cursor, &chs);
                self.cursor += 1;
            }
            Key::Backspace if self.cursor > 0 => {
                self.pick_hist();
                self.line.remove(self.cursor - 1);
                self.cursor -= 1;
            }
            Key::Delete if self.cursor < self.cur_line().len() => {
                self.pick_hist();
                self.line.remove(self.cursor);
            }
            Key::Left if self.cursor > 0 => self.cursor -= 1,
            Key::Right if self.cursor < self.cur_line().len() => self.cursor += 1,
            Key::Up if self.hist_pos > 0 => {
                self.hist_pos -= 1;
                self.cursor = self.cursor.clamp(0, self.cur_line().len());
            },
            Key::Down if self.hist_pos < self.history.len() => {
                self.hist_pos += 1;
                self.cursor = self.cursor.clamp(0, self.cur_line().len());
            },
            Key::Tab => {
                if self.autocomplete.is_empty() {
                    return TextInputRequest::Autocomplete;
                } else {
                    self.pick_hist();
                    self.line.insert_str(self.cursor, &self.autocomplete);
                    self.cursor += self.autocomplete.len();
                }
            }
            Key::Enter => {
                self.pick_hist();
                self.cursor = 0;
                self.autocomplete = String::new();
                let old_line = mem::replace(&mut self.line, String::new());
                self.tainted = true;
                if !old_line.trim().is_empty() {
                    if self.history.len() == self.hist_cap {
                        self.history.pop_front();
                    }
                    self.history.push_back(old_line.clone());
                }
                self.hist_pos = self.history.len();
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
            Action::KeyPress { key } if key.is_shift() => {
                self.shift = true;
                TextInputRequest::Nothing
            }
            Action::KeyRelease { key } if key.is_shift() => {
                self.shift = false;
                TextInputRequest::Nothing
            }
            Action::KeyPress { key } => self.keypress(key),
            _ => TextInputRequest::Nothing,
        }
    }

    /// Builds a `Vec<Text>` with the prompt line, for rendering.
    pub fn render<'s>(&mut self) -> Vec<Text> {
        self.tainted = false;
        let line = self.cur_line();
        if self.cursor == line.len() {
            if self.autocomplete.is_empty() {
                text![
                    "{}"(self.prompt),
                    bright_white "{}"(line),
                    bright_white underline " ",
                ]
            } else {
                text![
                    "{}"(self.prompt),
                    bright_white "{}"(line),
                    bright_black underline "{}"(&self.autocomplete[..1]),
                    bright_black "{}"(&self.autocomplete[1..]),
                ]
            }
        } else {
            if self.autocomplete.is_empty() {
                text![
                    "{}"(self.prompt),
                    bright_white "{}"(&line[..self.cursor]),
                    bright_white underline "{}"(&line[self.cursor..self.cursor+1]),
                    bright_white "{}"(&line[self.cursor+1..]),
                ]
            } else {
                text![
                    "{}"(self.prompt),
                    bright_white "{}"(&line[..self.cursor]),
                    bright_black underline "{}"(&self.autocomplete[..1]),
                    bright_black "{}"(&self.autocomplete[1..]),
                    bright_white "{}"(&line[self.cursor..]),
                ]
            }
        }
    }
}
