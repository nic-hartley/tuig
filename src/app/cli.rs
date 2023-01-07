use std::{
    sync::{Arc, RwLock},
    collections::{HashMap, VecDeque},
};

use crate::{
    agents::{
        tools::{autocomplete_with, Tool},
        Bundle, Event,
    },
    io::{
        clifmt::Text,
        input::{Action, Key},
        output::Screen,
    },
    text, GameState, Machine,
};

use super::App;

const MAX_SCROLL_LINES: usize = 1000;

/// The high-level state of the CLI, for passing to commands.
///
/// Note this is not updated live; it's the state of the CLI as of whenever the command was run.
#[derive(Clone, Default)]
pub struct CliState {
    /// The machine currently logged into
    pub machine: Arc<RwLock<Machine>>,
    /// The current working directory of the CLI
    pub cwd: String,
}

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
    /// the tools available at the command line
    tools: HashMap<String, Box<dyn Tool>>,
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
            tools: Default::default(),
        }
    }
}

impl CliApp {
    fn add_scroll(&mut self, line: Vec<Text>) {
        if self.scroll.len() == MAX_SCROLL_LINES {
            self.scroll.pop_front();
        }
        self.scroll.push_back(line.clone());
        self.unread += 1;
    }

    fn run_cmd(&mut self, events: &mut Vec<Event>) {
        self.add_scroll(text!("> ", bright_white "{}"(self.line), "\n"));
        self.history.push(self.line.clone());
        let line = self.line.trim();
        if line.is_empty() {
            return;
        }
        let (cmd, rest) = match line.split_once(' ') {
            Some(p) => p,
            None => (line, ""),
        };
        if let Some(tool) = self.tools.get(cmd) {
            let mut machine = Machine::default();
            machine.files.insert("awoo".into(), "".into());
            machine.files.insert("awful".into(), "".into());
            machine.files.insert("thingy".into(), "".into());
            machine.files.insert("machomp".into(), "".into());
            machine.files.insert("stuff/foo1".into(), "".into());
            machine.files.insert("stuff/foo2".into(), "".into());
            let cli_state = CliState {
                machine: Arc::new(RwLock::new(machine)),
                // TODO: track a real CWD
                cwd: "".into(),
            };
            events.push(Event::SpawnAgent(Bundle::of(
                tool.run(rest.trim(), &cli_state),
            )));
            self.prompt = false;
        } else {
            self.add_scroll(
                text![bright_red "ERROR", ": Command ", bright_white "{}"(cmd), " not found.\n"],
            );
        }
    }

    fn autocomplete(&self, line: &str) -> String {
        let mut machine = Machine::default();
        machine.files.insert("awoo".into(), "".into());
        machine.files.insert("awful".into(), "".into());
        machine.files.insert("thingy".into(), "".into());
        machine.files.insert("machomp".into(), "".into());
        machine.files.insert("stuff/foo1".into(), "".into());
        machine.files.insert("stuff/foo2".into(), "".into());
        let cli_state = CliState {
            machine: Arc::new(RwLock::new(machine)),
            // TODO: track a real CWD
            cwd: "".into(),
        };
        if let Some((cmd, rest)) = line.split_once(char::is_whitespace) {
            if let Some(tool) = self.tools.get(cmd) {
                tool.autocomplete(rest.trim_start(), &cli_state)
            } else {
                String::new()
            }
        } else {
            autocomplete_with(line, self.tools.keys())
        }
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
                    self.autocomplete = self.autocomplete(&self.line[..self.cursor]);
                } else {
                    self.line.insert_str(self.cursor, &self.autocomplete);
                    self.cursor += self.autocomplete.len();
                    self.autocomplete = String::new();
                }
                return true;
            }
            Key::Enter => {
                self.cursor = 0;
                self.run_cmd(events);
                self.line.clear();
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
            if self.autocomplete.is_empty() {
                text![
                    "> ",
                    bright_white "{}"(self.line),
                    bright_white underline " ",
                ]
            } else {
                text![
                    "> ",
                    bright_white "{}"(self.line),
                    bright_black underline "{}"(&self.autocomplete[..1]),
                    bright_black "{}"(&self.autocomplete[1..]),
                ]
            }
        } else {
            if self.autocomplete.is_empty() {
                text![
                    "> ",
                    bright_white "{}"(&self.line[..self.cursor]),
                    bright_white underline "{}"(&self.line[self.cursor..self.cursor+1]),
                    bright_white "{}"(&self.line[self.cursor+1..]),
                ]
            } else {
                text![
                    "> ",
                    bright_white "{}"(&self.line[..self.cursor]),
                    bright_black underline "{}"(&self.autocomplete[..1]),
                    bright_black "{}"(&self.autocomplete[1..]),
                    bright_white "{}"(&self.line[self.cursor..]),
                ]
            }
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
            Event::CommandDone => {
                self.prompt = true;
                true
            }
            Event::InstallTool(tool) => {
                let tool = tool.take().expect("Tool taken by something other than CLI");
                self.tools.insert(tool.name().into(), tool);
                false
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
        screen.textbox(main_text).pos(0, 1).height(main_text_height).scroll_bottom(true);
    }
}
