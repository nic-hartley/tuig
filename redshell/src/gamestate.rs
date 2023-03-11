use std::sync::Arc;

use crate::{app::App, machine::Machine};

/// The current state of the game, including the state of the UI.
#[derive(Default)]
pub struct GameState {
    /// The player's name, of course
    pub player_name: String,
    /// The apps currently available (in order of tabs)
    pub apps: Vec<Box<dyn App>>,
    /// The player's computer
    pub machine: Arc<Machine>,
}

impl GameState {
    pub fn for_player(name: String) -> Self {
        Self {
            player_name: name,
            apps: Default::default(),
            machine: Arc::new(Default::default()),
        }
    }
}
