use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};

use crate::{
    agents::{
        tools::{autocomplete_with, Tool},
        Bundle, Event,
    },
    io::{
        clifmt::Text,
        input::Action,
        output::Screen,
        text_input::{TextInput, TextInputRequest},
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
    pub machine: Arc<Machine>,
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
    /// the text input players type into
    input: TextInput,
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
            input: TextInput::new("> "),
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

    fn run_cmd(&mut self, line: String, events: &mut Vec<Event>) {
        self.add_scroll(text!("> ", bright_white "{}"(line), "\n"));
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return;
        }
        let (cmd, rest) = match trimmed.split_once(' ') {
            Some(p) => p,
            None => (trimmed, ""),
        };
        if let Some(tool) = self.tools.get(cmd) {
            let mut machine = Machine::default();
            machine.write("/awoo", "".into()).unwrap();
            machine.write("/awful", "".into()).unwrap();
            machine.write("/thingy", "".into()).unwrap();
            machine.write("/machomp", "".into()).unwrap();
            machine.write("/stuff/foo1", "".into()).unwrap();
            machine.write("/stuff/foo2", "".into()).unwrap();
            let cli_state = CliState {
                machine: Arc::new(machine),
                // TODO: track a real CWD
                cwd: "/".into(),
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
        self.history.push(line);
    }

    fn autocomplete(&self, line: &str) -> String {
        let mut machine = Machine::default();
        machine.write("/awoo", "".into()).unwrap();
        machine.write("/awful", "".into()).unwrap();
        machine.write("/thingy", "".into()).unwrap();
        machine.write("/machomp", "".into()).unwrap();
        machine.write("/stuff/foo1", "".into()).unwrap();
        machine.write("/stuff/foo2", "".into()).unwrap();
        let cli_state = CliState {
            machine: Arc::new(machine),
            // TODO: track a real CWD
            cwd: "/".into(),
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
}

impl App for CliApp {
    fn name(&self) -> &'static str {
        "terminal"
    }

    fn input(&mut self, a: Action, events: &mut Vec<Event>) -> bool {
        if self.prompt {
            match self.input.action(a) {
                TextInputRequest::Nothing => (),
                TextInputRequest::Autocomplete => {
                    let complete = self.autocomplete(self.input.completable());
                    self.input.set_complete(complete);
                }
                TextInputRequest::Line(l) => {
                    self.run_cmd(l, events);
                }
            };
            self.input.tainted()
        } else {
            false
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
            .chain(if self.prompt {
                self.input.render()
            } else {
                vec![]
            })
            .collect::<Vec<_>>();
        let main_text_height = screen.size().y() - help_height;
        screen
            .textbox(main_text)
            .pos(0, 1)
            .height(main_text_height)
            .scroll_bottom(true);
    }
}
