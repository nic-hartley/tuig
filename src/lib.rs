#![cfg_attr(coverage, feature(no_coverage))]

use machine::Machine;

pub mod agents;
pub mod app;
pub mod constants;
pub mod cutscenes;
pub mod game;
pub mod io;
pub mod machine;
pub mod tools;
mod util;
mod timing;

/// The current state of the game, including the state of the UI.
#[derive(Default)]
pub struct GameState {
    /// The player's name, of course
    pub player_name: String,
    /// The player's computer
    pub machine: Machine,
}
