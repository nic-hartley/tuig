use std::{collections::VecDeque, mem};

use crate::{
    agents::{Event, tools::AutocompleteType},
    io::{
        clifmt::Text,
        input::{Action, Key},
        output::Screen,
    },
    text, GameState,
};

use super::App;

const MAX_SCROLL_LINES: usize = 1000;

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct CliApp {
    /// prior commands, as entered by the player (for scrolling through with the up arrow)
    history: Vec<String>,
    /// prior lines of output (for rendering, and limited to ~MAX_SCROLL_LINES lines, depending on word wrap)
    scroll: VecDeque<Vec<Text>>,
    /// whether the prompt is currently visible
    prompt: bool,
    /// line(s) currently being typed
    line: String,
    /// cursor position in the line being typed
    cursor: usize,
    /// any autocomplete that's been requested
    autocomplete: String,
    /// help text
    help: String,
    /// lines of output that haven't been read yet
    unread: usize,
}

impl Default for CliApp {
    fn default() -> Self {
        Self {
            history: Default::default(),
            scroll: Default::default(),
            prompt: true,
            line: Default::default(),
            cursor: Default::default(),
            autocomplete: Default::default(),
            help: Default::default(),
            unread: Default::default(),
        }
    }
}

impl CliApp {
    fn add_scroll(&mut self, line: Vec<Text>) {
        if self.scroll.len() == MAX_SCROLL_LINES {
            self.scroll.pop_back();
        }
        self.scroll.push_front(line.clone());
        self.unread += 1;
    }

    fn run_cmd(&mut self, line: String, events: &mut Vec<Event>) {
        self.add_scroll(text!("> ", bright_white "{}"(line)));
        events.push(Event::output(text!("command run: {}"(line))));
        events.push(Event::CommandDone);
        self.history.push(line);
    }

    fn autocomplete(&self, line: &str) -> String {
        AutocompleteType::Choices(vec!["opt1".into(), "opt2".into(), "help".into()]).complete(line, &Default::default())
    }

    fn keypress(&mut self, key: Key, events: &mut Vec<Event>) -> bool {
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
                    self.autocomplete = self.autocomplete(&self.line);
                } else {
                    self.line.insert_str(self.cursor, &self.autocomplete);
                    self.cursor += self.autocomplete.len();
                    self.autocomplete = String::new();
                }
                return true;
            }
            Key::Enter => {
                self.cursor = 0;
                let line = mem::replace(&mut self.line, String::new());
                self.run_cmd(line, events);
            }
            _ => return false,
        }
        self.autocomplete = String::new();
        true
    }

    fn prompt_line(&self) -> Vec<Text> {
        if !self.prompt {
            return vec![];
        }
        if self.cursor == self.line.len() {
            let cursor_ch = if self.autocomplete.is_empty() { " " } else { &self.autocomplete[..1] };
            let rest = if self.autocomplete.len() < 1 { "" } else { &self.autocomplete[1..] };
            text!(
                "> ",
                bright_white "{}"(self.line),
                underline bright_black "{}"(cursor_ch),
                bright_black "{}"(rest),
            )
        } else {
            let pre_cursor = &self.line[..self.cursor];
            let cursor_ch = &self.line[self.cursor..self.cursor+1];
            let post_cursor = &self.line[self.cursor+1..];
            text!(
                "> ",
                bright_white "{}"(pre_cursor),
                bright_white underline "{}"(cursor_ch),
                bright_white "{}"(post_cursor),
                bright_black "{}"(self.autocomplete),
            )
        }
    }
}

impl App for CliApp {
    fn name(&self) -> &'static str {
        "terminal"
    }

    fn input(&mut self, a: Action, events: &mut Vec<Event>) -> bool {
        match a {
            Action::KeyPress { key } => self.keypress(key, events),
            _ => false,
        }
    }

    fn on_event(&mut self, ev: &Event) -> bool {
        match ev {
            Event::CommandOutput(line) => {
                self.add_scroll(line.clone());
                true
            }
            _ => false,
        }
    }

    fn notifs(&self) -> usize {
        self.unread
    }

    fn render(&mut self, _state: &GameState, screen: &mut Screen) {
        self.unread = 0;

        let help_height = if !self.help.is_empty() {
            let tb_met = screen
                .textbox(text!(bright_green "# {}"(self.help)))
                .scroll_bottom(true)
                .indent(2)
                .first_indent(0)
                .render();
            tb_met.lines
        } else {
            0
        };

        let main_text = self
            .scroll
            .iter()
            .flat_map(|v| v)
            .cloned()
            .chain(self.prompt_line())
            .collect::<Vec<_>>();
        let main_text_height = screen.size().y() - help_height;
        screen.textbox(main_text).pos(0, 1).height(main_text_height);
    }
}
