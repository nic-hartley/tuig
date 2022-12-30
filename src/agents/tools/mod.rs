use crate::GameState;

use super::Agent;

mod args;
pub use args::{AutocompleteType, BsdCompleter};

pub trait Tool {
    fn autocomplete(&self, line: &str, state: &GameState) -> String;
    fn run(&self, line: &str) -> Box<dyn Agent>;
}
