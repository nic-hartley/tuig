//! The `Game` and the `Response` it can give the engine.

use tuig_iosys::ui::Region;

use crate::{Message, Replies};

/// How a `Game` can respond to inputs or messages, affecting the whole game.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Response {
    /// Nothing in particular needs to be done.
    Nothing,
    /// The visual state has updated, and the [`Screen`] needs to be redrawn.
    Redraw,
    /// The game should be exited, e.g. because the user clicked "Exit" in the menu.
    Quit,
}

/// Represents a game which can be run in the main loop.
///
/// Note that `Game`s don't run the bulk of the game logic; that's the `Agent`'s job. The `Game` trait is the place
/// where user input and rendering happen. The idea here is:
///
/// - When there's relevant user input, you can send messages or make new agents, and/or update state for rendering
/// - When a message happens (including one this `Game` spawned!), you can update internal state for rendering
/// - You *don't* react to messages with more messages -- that's an `Agent`'s job
/// - Come time to render, you already have all the info you need from previous inputs/messages
///
/// This is a fairly typical Elm-style GUI, though obviously the message bus is used for more than just UI messages,
/// as it's also the primary method of communication between agents and the game. This makes the code a bit harder to
/// write, but it clearly separates concerns and helps you put heavy logic somewhere other than the render thread, and
/// ideally split it into multiple `Agent`s so it can be parallelized neatly.
pub trait Game: Send {
    /// The message that this `Game` will be passing around between `Agent`s and itself.
    type Message: Message;

    /// A message has happened; update the UI accordingly.
    fn message(&mut self, message: &Self::Message) -> Response;

    /// Attach the game to a [`Region`] occupying the whole screen. Based on the inputs given, re-render the player's
    /// UI, and inform [`Agent`](crate::Agent)s accordingly.
    ///
    /// If you want to render in terms of a raw [`Screen`](tuig_iosys::Screen) and input [`Action`](tuig_iosys::Action)
    /// instead, call [`Region::attach`] with a [`RawAttachment`](tuig_iosys::ui::attachments::RawAttachment).
    fn attach<'s>(&mut self, into: Region<'s>, replies: &mut Replies<Self::Message>);
}
