pub mod app;
pub mod constants;
pub mod event;
pub mod io;
mod util;

/// The current state of the game, including the state of the UI.
#[derive(PartialEq, Eq, Clone, Debug, Default)]
pub struct GameState {
    /// The player's name, of course
    pub player_name: String,
    /// The apps currently available (in order of tabs)
    pub apps: Vec<app::Apps>,
}
