//! tuig is a game engine hyperfocused on systemic textmode games.
//! 
//! If you want to jump straight in with an example, check [`docs::walkthrough`].
//! 
//! # Architecture
//! 
//! tuig is built around a shared event bus. Everything that happens in the game is represented by a single type you
//! define that extends [`Message`]. You'll also have a lot of different types of [`Agent`]s, which do all of your
//! relevant simulation and event processing. The thing the player actually interacts with is the [`Game`], which
//! processes user input and produces the output, and communicates with agents by spawning events. Click those links
//! for more details on each specific trait.
//! 
//! `Agent`s and the `Game` (coincidentally the name of my new ska band) can inject new agents or messages through
//! [`Replies`], which is the general handle into the game's internals for most things. An `Agent` can put itself to
//! sleep -- including metaphorically -- by returning different [`ControlFlow`]s, which is useful for boosting
//! performance by reducing how many agents need to be considered but shouldn't be relied on for anything too serious.
//! `Game`s have a similar option in [`Response`], though they're gonna get called constantly anyway, so their
//! responses are about re-rendering the text or exiting the game entirely.
//! 
//! # Feature selection
//! 
//! Before you can actually *run* your game, you need to select some backend functionality with Cargo features.
//! 
//! The runner is what actually passes messages to each `Agent` and the `Game`, and coordinates the IO system with all
//! that. Pick one of:
//! 
//! -   `run_rayon`: A very reasonable default. Distributes agents across multiple threads with `rayon`, which is very
//!     good at making good use of all available cores for this sort of thing.
//! -   `run_single`: Useful only if `rayon` is undesirable for some reason, e.g. small-scale unit tests or `no_std`.
//! 
//! The IO system -- which of the [`tuig-iosys::backends`](https://docs.rs/tuig-iosys/latest/tuig_iosys/backends) --
//! will be handling our platform input and output. All the `tuig-iosys` backends are available, but the features
//! have an extra `io_` prefix. So you can have:
//! 
//! -   `io_nop`: Great for integration tests where you don't actually care about input or output, but want an
//!     otherwise complete tuig.
//! -   `io_cli_crossterm`: Render the character grid to a real terminal, using `crossterm`.
//! -   `io_gui_softbuffer`: Render the character grid to a `winit` window, using CPU rendering with `softbuffer`.
//!     It's very widely compatible, because you literally don't need any 3D hardware for it to work.
//! 
//! You have to pick exactly one runner, but you can choose more than one IO system. [`Runner::load_run`] will try to
//! intelligently pick "the best it can" given the ones you've turned on, but if you very reasonably disagree, you can
//! load your preferred system and use [`Runner::run`] instead.

mod agent;
mod game;
mod runner;
mod util;
#[cfg(doc)]
pub mod docs;

pub use {
    agent::{Agent, ControlFlow, WaitHandle},
    game::{Game, Message, Replies, Response},
    runner::Runner,
    tuig_iosys as io,
};

/// A shortcut for basic games which only need to be started on the default, [`load!`](tuig_iosys::load!)ed IO system.
///
/// If you need more control, e.g. selecting a specific backend or queueing initial agents, use [`Runner`]. Many real
/// games won't, as they won't be doing anything until after some user input asks them to.
#[cfg(feature = "__io")]
pub fn run<G: Game + 'static>(game: G) -> G {
    Runner::new(game).load_run()
}
