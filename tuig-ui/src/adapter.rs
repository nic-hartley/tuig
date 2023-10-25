use tuig_iosys::{Screen, IoSystem, XY, Action, Result};

use crate::{attachments::Attachment, Region};

/// A convenience wrapper to make it easier to manage screens, actions, and input.
/// 
/// This is a very simple wrapper. You put in your desired `IoSystem`, and then you can call [`Self::input`],
/// [`Self::poll_input`], and [`Self::draw`] to run things. Internally it stores a [`Screen`], and it manages that
/// in a simple, allocation-avoiding/reusing way.
/// 
/// This is mean to handle average use-cases. You might well need to do something more specific -- and hopefully, this
/// at least serves as a jumping-off point.
pub struct Adapter<IO: IoSystem> {
    io: IO,
    screen: Screen,
}

impl<IO: IoSystem> Adapter<IO> {
    /// Create a new [`Adapter`] that will be adapting inputs and outputs from this [`IoSystem`].
    pub fn new(io: IO) -> Self {
        Self { io, screen: Screen::new(XY(0, 0)) }
    }

    /// Wait for one input like [`IoSystem::input`], then use it to render the attachment to the stored screen.
    /// 
    /// This returns an error if the `IoSystem` did, or otherwise whatever the root attachment does.
    pub fn input<'s, A: Attachment<'s>>(&'s mut self, root: A) -> Result<A::Output> {
        self.io.input().map(|input| self.feed(root, input))
    }

    /// As [`IoSystem::poll_input`] is to [`IoSystem::input`], this is to [`Self::input`] -- i.e. a non-blocking
    /// version of the same thing, which returns `Ok(None)` if there's no input to process.
    /// 
    /// Note that the attachment is consumed either way. Types implementing [`Attachment`] are meant to be ephemeral
    /// and cheap to produce -- see that type's docs for more info.
    pub fn poll_input<'s, A: Attachment<'s>>(&'s mut self, root: A) -> Result<Option<A::Output>> {
        self.io.poll_input().map(|o| o.map(|input| self.feed(root, input)))
    }

    /// Manually feed in an action, doing everything as though it was taken from the `IoSystem`.
    /// 
    /// This is probably most obviously useful for `adapter.feed(&mut attachment, Action::Redraw)` shortly before
    /// calling [`Self::draw`].
    pub fn feed<'s, A: Attachment<'s>>(&'s mut self, root: A, input: Action) -> A::Output {
        self.screen.resize(self.io.size());
        let region = Region::new(&mut self.screen, input);
        region.attach(root)
    }

    /// Draw the stored screen to the display.
    /// 
    /// This will **not** re-render anything if the screen isn't the right size -- it'll just try to draw. See the
    /// caveats in [`IoSystem::draw`] for more info.
    pub fn draw(&mut self) -> Result<()> {
        self.io.draw(&self.screen)
    }
}
