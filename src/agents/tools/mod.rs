use std::mem;

use crate::{app::CliState, io::clifmt::Text};

use super::{Agent, ControlFlow, Event};

mod args;
pub use args::{autocomplete, AutocompleteType, BsdArgs};

mod ls;
pub use ls::Ls;
mod touch;
pub use touch::Touch;
mod mkdir;
pub use mkdir::Mkdir;
mod cd;
pub use cd::Cd;

/// Common interface for all CLI tools
pub trait Tool {
    /// The name of the tool. This must be constant and identical for all tools of this type.
    /// 
    /// This is what the CLI uses to map invoked commands to the correct tool.
    fn name(&self) -> &'static str;
    /// Attempt to perform autocompletion, given the line up to the cursor location.
    fn autocomplete(&self, prefix: &str, state: &CliState) -> String;
    /// Create an agent to make this command have effects
    fn run(&self, line: &str, state: &CliState) -> Box<dyn Agent>;
}

/// [`Agent`] implementation that just outputs some fixed text, signals the CLI it's done, and dies.
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
