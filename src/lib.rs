use app::App;
use machine::Machine;

pub mod agents;
pub mod app;
pub mod constants;
pub mod cutscenes;
pub mod io;
pub mod machine;
pub mod tools;
mod util;

/// The current state of the game, including the state of the UI.
#[derive(Default)]
pub struct GameState {
    /// The player's name, of course
    pub player_name: String,
    /// The apps currently available (in order of tabs)
    pub apps: Vec<Box<dyn App>>,
    /// The player's computer
    pub machine: Machine,
}
