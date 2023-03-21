mod agent;
mod game;
mod runner;
mod util;

pub use {
    agent::{Agent, ControlFlow, WaitHandle},
    game::{Game, Message, Replies, Response},
    runner::Runner,
    tuig_iosys as io,
};

/// A shortcut for basic games which only need to be started on the default,
/// [`load!`](tuig_iosys::load)ed IO system.
///
/// If you need more control, e.g. selecting a specific backend or queueing
/// initial agents, use [`Runner`]. Many real games won't, as they won't be
/// doing anything until after some user input asks them to.
pub fn run<G: Game + 'static>(game: G) -> G {
    Runner::new(game).load_run()
}
