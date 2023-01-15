use std::mem;

use crate::{app::CliState, io::clifmt::Text};

use super::{Agent, ControlFlow, Event};

mod args;
pub use args::{autocomplete_with, AutocompleteType, BsdCompleter};

mod ls;
pub use ls::Ls;
mod touch;
pub use touch::Touch;
mod mkdir;
pub use mkdir::Mkdir;
mod cd;
pub use cd::Cd;

pub trait Tool {
    fn name(&self) -> &'static str;
    fn autocomplete(&self, line: &str, state: &CliState) -> String;
    fn run(&self, line: &str, state: &CliState) -> Box<dyn Agent>;
}

/// If a tool just needs to output based on the game state and doesn't actually do any processing, this makes that
/// easy. It implements `Agent` just to output the lines, then signal that it's done.
struct FixedOutput(Vec<Vec<Text>>);

impl Agent for FixedOutput {
    fn start(&mut self, replies: &mut Vec<super::Event>) -> ControlFlow {
        let lines = mem::take(&mut self.0);
        replies.extend(lines.into_iter().map(|l| Event::CommandOutput(l)));
        replies.push(Event::CommandDone);
        ControlFlow::Kill
    }

    fn react(&mut self, _events: &[Event], _replies: &mut Vec<Event>) -> ControlFlow {
        ControlFlow::Kill
    }
}

/// An agent which does nothing and immediately dies.
// Big mood, buddy.
pub struct NoOutput;
impl Agent for NoOutput {
    fn start(&mut self, replies: &mut Vec<Event>) -> ControlFlow {
        replies.push(Event::CommandDone);
        ControlFlow::Kill
    }

    fn react(&mut self, _events: &[Event], _replies: &mut Vec<Event>) -> ControlFlow {
        ControlFlow::Kill
    }
}
