use std::{collections::VecDeque, sync::Arc};

use crate::{
    agents::{
        tools::{autocomplete, Tool},
        Bundle, Event,
    },
    io::{
        clifmt::Text,
        helpers::{TextInput, TextInputRequest},
        input::Action,
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
    pub machine: Arc<Machine>,
    /// The current working directory of the CLI
    pub cwd: String,
}

pub struct CliApp {
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

    /// The current state of the CLI
    state: CliState,
}

impl Default for CliApp {
    fn default() -> Self {
        Self {
            scroll: Default::default(),
            prompt: true,
            input: TextInput::new("> ", 100),
            help: Default::default(),
            unread: Default::default(),
            state: CliState {
                machine: Default::default(),
                cwd: "/".into(),
            },
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
        if let Some(tool) = self.state.machine.tools.get(cmd).map(|r| r.value().clone()) {
            events.push(Event::SpawnAgent(Bundle::of(
                tool.run(rest.trim(), &self.state),
            )));
            self.prompt = false;
        } else {
            let line =
                text![bright_red "ERROR", ": Command ", bright_white "{}"(cmd), " not found.\n"];
            self.add_scroll(line);
        }
    }

    fn autocomplete(&self, line: &str) -> String {
        if let Some((cmd, rest)) = line.split_once(char::is_whitespace) {
            if let Some(tool) = self.state.machine.tools.get(cmd) {
                tool.autocomplete(rest.trim_start(), &self.state)
            } else {
                String::new()
            }
        } else {
            struct RefMultiAdapter<'a>(
                dashmap::mapref::multiple::RefMulti<'a, String, Arc<dyn Tool>>,
            );
            impl<'a> AsRef<str> for RefMultiAdapter<'a> {
                fn as_ref(&self) -> &str {
                    self.0.key().as_ref()
                }
            }
            autocomplete(line, self.state.machine.tools.iter().map(RefMultiAdapter))
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
                self.state
                    .machine
                    .tools
                    .insert(tool.name().into(), tool.into());
                false
            }
            Event::ChangeDir(new_dir) => {
                // we blindly trust that whoever sent that event knew what they were doing
                self.state.cwd = new_dir.to_owned();
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
