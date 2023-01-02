use std::mem;

use crate::{GameState, io::clifmt::Text};

use super::{Agent, Event, ControlFlow};

mod args;
pub use args::{autocomplete_with, AutocompleteType, BsdCompleter};

mod ls;
pub use ls::Ls;

pub trait Tool {
    fn name(&self) -> &'static str;
    fn autocomplete(&self, line: &str, state: &GameState) -> String;
    fn run(&self, line: &str, state: &GameState) -> Box<dyn Agent>;
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
