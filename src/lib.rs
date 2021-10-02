pub mod io;
pub mod app;
mod util;

/// The current state of the game, including the state of the UI.
pub struct GameState {
    /// The player's name, of course
    player_name: String,
    /// The apps currently available (in order of tabs)
    apps: Vec<app::Apps>,
}
