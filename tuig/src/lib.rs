//! tuig is a game engine hyperfocused on systemic textmode games.
//!
//! If you want to jump straight in with an example, check [`docs::walkthrough`].
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
//! The IO system -- one of the [`tuig-iosys::backends`](tuig_iosys::backends) -- will be handling our platform input
//! and output. All the `tuig-iosys` backends are available, but the features have an extra `io_` prefix. So you have:
//!
//! -   `io_nop`: Great for integration tests where you don't actually care about input or output, but want an
//!     otherwise complete tuig.
//! -   `io_cli_crossterm`: Render the character grid to a real terminal, using `crossterm`.
//! -   `io_gui_softbuffer`: Render the character grid to a `winit` window, using CPU rendering with `softbuffer`.
//!     It's very widely compatible, because you literally don't need any 3D hardware for it to work.
//!
//! You have to pick exactly one runner, but you can choose more than one IO system. [`Runner::load_run`] will try to
//! intelligently pick "the best it can" given the ones you've turned on, but if you very reasonably disagree, or if
//! you want a third-party backend, you can load your preferred system and use [`Runner::run`] instead.
//!
//! # Architecture
//!
//! tuig is built around a shared event bus. Everything that happens in the game is represented by a single type 
//! <code>M: [Message]</code>. You'll also have a variety of of [`Agent<M>`]s, which do all of your actual simulation
//! and event processing. The thing the player actually interacts with is the [`Game<M>`], which processes user input
//! and renders the output, and communicates with agents by spawning events. Click those links for more details on
//! each specific trait.
//! 
//! Events passed around are loosely organized into "rounds". As events are passed to agents, their [`Replies`] are
//! collected, then applied only after each agent has seen the current round of events. However, agents **aren't
//! necessarily** running in lockstep. They don't even all necessarily see the same rounds! You'll occasionally see
//! them mentioned because that's how the engine works internally, and it explains why certain things happen or don't.
//!
//! `Agent`s and the `Game` (coincidentally the name of my new ska band) can inject new agents or messages through
//! [`Replies`], which is the general handle into the game's internals for most things. An `Agent` can put itself to
//! sleep -- including metaphorically -- by returning different [`ControlFlow`]s, which is useful for boosting
//! performance by reducing how many agents need to be considered but shouldn't be relied on for anything too serious.
//! `Game`s have a similar option in [`Response`], though they're gonna get called constantly anyway, so their
//! responses are about re-rendering the text or exiting the game entirely.

#![cfg_attr(doc, feature(doc_cfg, doc_auto_cfg))]
tuig_pm::force_docs_nightly!();

mod agent;
#[cfg(doc)]
pub mod docs;
mod game;
mod runner;
mod util;
mod message;

pub use {
    agent::{Agent, ControlFlow, WaitHandle},
    game::{Game, Response},
    message::{Message, Replies},
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
