use std::collections::HashMap;

use app::App;

pub mod agents;
pub mod app;
pub mod constants;
pub mod cutscenes;
pub mod io;
mod util;

/// The current state of the game, including the state of the UI.
#[derive(Default)]
pub struct GameState {
    /// The player's name, of course
    pub player_name: String,
    /// The apps currently available (in order of tabs)
    pub apps: Vec<Box<dyn App>>,
    /// The files on the virtual machine
    pub files: HashMap<String, Vec<u8>>,
}

impl GameState {
    pub fn for_player(name: String) -> Self {
        Self {
            player_name: name,
            apps: Default::default(),
            files: Default::default(),
        }
    }
}
