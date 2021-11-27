use crate::{GameState, io::{Action, Screen}};

/// Each app is a single tab in the game's window view, e.g. chat. They exclusively handle IO: Processing user input
/// and rendering (part of) game state.
///
/// Apps are only given input and rendered when they're on-screen, but they receive all events. Note, though, that
/// events may be batched when the app is offscreen, so that systems and the onscreen app can be updated on time.
#[enum_dispatch::enum_dispatch]
pub trait App {
    // TODO: Replace String with actual events

    /// The name of this app's tab in the header. (should be constant, hence &'static)
    fn name(&self) -> &'static str;

    /// Take a single input action, returning any new events generated as a result.
    fn input(&mut self, a: Action) -> Vec<String>;
    /// Receive at least one event, to update the rendered game state.
    fn on_event(&mut self, evs: &[String]);

    /// The number of notifications this app has.
    fn notifs(&self) -> usize;
    /// Display the game state on screen.
    fn render(&self, state: &GameState, screen: &mut dyn Screen);
}

mod chat;
pub use chat::ChatApp;

#[enum_dispatch::enum_dispatch(App)]
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Apps {
    ChatApp,
}
