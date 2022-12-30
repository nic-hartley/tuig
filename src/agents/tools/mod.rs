use crate::GameState;

use super::Agent;

mod completers;
pub use completers::AutocompleteType;

pub trait Tool {
    fn autocomplete(&self, line: &str, state: &GameState) -> String;
    fn run(&self, line: &str) -> Box<dyn Agent>;
}
