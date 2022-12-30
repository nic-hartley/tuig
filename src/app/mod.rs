use crate::{
    agents::Event,
    io::{input::Action, output::Screen},
    GameState,
};

/// Each app is a single tab in the game's window view, e.g. chat. They exclusively handle IO: Processing user input
/// and rendering (part of) game state.
///
/// Apps are only given input and rendered when they're on-screen, but they receive all events. Note, though, that
/// events may be batched when the app is offscreen, so that systems and the onscreen app can be updated on time.
pub trait App {
    /// The name of this app's tab in the header. (should be constant, hence &'static)
    fn name(&self) -> &'static str;

    /// Take a single input action, returning any new events generated as a result.
    ///
    /// Returns `true` if it will need to be redrawn, or `false` otherwise.
    fn input(&mut self, a: Action, events: &mut Vec<Event>) -> bool;
    /// Receive an event, in case the app needs to care to render it.
    ///
    /// Returns `true` if it will need to be redrawn, or `false` otherwise.
    fn on_event(&mut self, ev: &Event) -> bool;

    /// The number of notifications this app has.
    fn notifs(&self) -> usize;
    /// Display the game state on screen.
    ///
    /// You can be sure that this will never be called except when the module is the active one; feel free to use it
    /// for e.g. clearing notifications.
    fn render(&mut self, state: &GameState, screen: &mut Screen);
}

mod chat;
pub use chat::ChatApp;
mod cli;
pub use cli::CliApp;
