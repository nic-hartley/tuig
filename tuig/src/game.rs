//! The `Game` and the `Response` it can give the engine.

use tuig_ui::Region;

use crate::{Message, Replies};

/// Represents a game which can be run in the main loop.
///
/// Note that `Game`s don't run the bulk of the game logic; that's the `Agent`'s job. The `Game` trait is the place
/// where user input and rendering happen. The idea here is:
///
/// - When there's relevant user input or just periodically, you can send messages, make new agents, etc. and render.
/// - When you receive a message, you can update your internal state for your next render and user input.
///
/// This uses the usual [`tuig_ui`] system: You get a [`Region`], you attach something into it, etc. But you
/// deliberately can't trigger arbitrary messages from receiving messages: That's an `Agent`'s job. It could make some
/// for simpler code to let it happen in `Game`, but it also makes it very easy to stall out your render thread.
pub trait Game: Send {
    /// The message that this `Game` will be passing around between `Agent`s and itself.
    type Message: Message;

    /// A message has happened; update the UI accordingly.
    fn message(&mut self, message: &Self::Message);

    /// Attach the game to a [`Region`] occupying the whole screen. Based on the inputs given, re-render the player's
    /// UI, and inform [`Agent`](crate::Agent)s accordingly. This always gets called at least once per frame, either
    /// when user input happens or with `Redraw` if there was no input during the frame.
    ///
    /// If you want to render in terms of a raw [`Screen`](tuig_iosys::Screen) and input [`Action`](tuig_iosys::Action)
    /// instead, call [`Region::attach`] with a [`RawAttachment`](tuig_ui::attachments::RawAttachment).
    ///
    /// This will blindly pass inputs through to you -- be sure to check for `Closed` events, perform the cleanup you
    /// need to do, and return `true` as appropriate. (If cleanup might take a while, e.g. saving the game, consider
    /// spawning an agent to do it.)
    ///
    /// Return `true` to completely exit the game, e.g. if the player pressed a "Quit" button in the menu.
    fn attach(&mut self, into: Region<'_>, replies: &mut Replies<Self::Message>) -> bool;
}
