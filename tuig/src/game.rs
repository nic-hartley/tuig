//! The `Game` and the `Response` it can give the engine.

use tuig_iosys::{Action, Screen};

use crate::{Message, Replies};

/// Allows a [`Game`] to control the engine in response to events or input.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Response {
    /// Nothing in particular needs to be done.
    Nothing,
    /// The visual state has updated, and the screen needs to be redrawn.
    Redraw,
    /// The game should be exited, e.g. because the user clicked "Exit" in the menu.
    Quit,
}

/// Represents a game which can be run in the main loop.
///
/// Note that `Game`s don't run the bulk of the game logic; that's the `Agent`'s job. The `Game` trait is the place
/// where user input and rendering happen. The idea here is:
///
/// - When there's relevant user input, you can send `Message`s or make new agents, and/or update state for rendering
/// - When a `Message` happens (including one you spawned!), you can update internal state for rendering
/// - You *don't* react to messages with more messages -- that's an `Agent`'s job
/// - Come time to render, you already have all the info you need from previous inputs/events
///
/// This is a fairly typical Elm-style GUI, though obviously the event bus is used for more than just events, as it's
/// also the primary method of communication between agents and the game. This makes the code a bit harder to write,
/// but it clearly separates concerns and encourages you to put heavy logic somewhere other than the render thread.
pub trait Game: Send {
    /// The message that this `Game` will be passing around between `Agent`s and itself.
    type Message: Message;

    /// The user has done some input; update the UI and inform [`Agent`](crate::Agent)s accordingly.
    ///
    /// Returns whether the game needs to be redrawn after the user input.
    fn input(&mut self, input: Action, replies: &mut Replies<Self::Message>) -> Response;

    /// An event has happened; update the UI accordingly.
    ///
    /// Returns whether the game needs to be redrawn after the event.
    fn event(&mut self, event: &Self::Message) -> Response;

    /// Render the game onto the provided `Screen`.
    fn render(&self, onto: &mut Screen);
}
