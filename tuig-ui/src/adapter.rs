use std::time::{Duration, Instant};

use tuig_iosys::{Action, IoSystem, Result, Screen, XY};

use crate::{attachments::Attachment, Region};

/// A convenience wrapper to make it easier to manage screens, actions, and input in simple cases.
///
/// This is a very simple wrapper. You put in your desired `IoSystem`, and then you can call [`Self::input`],
/// [`Self::poll_input`], and [`Self::draw`] to run things. Internally it stores a [`Screen`], and it manages that
/// in a simple, allocation-avoiding/reusing way.
///
/// This is meant to handle simple, average use-cases. You might well need to do something more specific -- and
/// hopefully, this code is simple enough to serve as a jumping-off point. It includes functionality for:
///
/// - Only rendering if the screen has changed
/// - Capping framerates, optionally
///
/// The biggest benefit is that this API will stay far more stable than the "lower level" ones, even during this early
/// alpha phase, incorporating lots of [planned] [features] more or less seamlessly. The biggest drawback is that it
/// you can't precisely control when or how it does each step, which might be a dealbreaker for advanced use-cases.
/// But the code also tries to be easy enough to read that you can make your own.
pub struct Adapter<IO: IoSystem> {
    io: IO,
    old: Screen,
    current: Screen,
    fps: Option<(Duration, Instant)>,
}

impl<IO: IoSystem> Adapter<IO> {
    /// Create a new `Adapter` that will be adapting inputs and outputs from this [`IoSystem`], with no FPS cap.
    pub fn new(io: IO) -> Self {
        Self {
            io,
            old: Screen::new(XY(0, 0)),
            current: Screen::new(XY(0, 0)),
            fps: None,
        }
    }

    /// Set an FPS cap on this `Adapter`.
    ///
    /// Remember this just skips calls to [`Self::draw`] when they're too fast, so your framerate will probably be a
    /// bit lower than this. If you need to minimize frame times, you'll need to implement more complex logic around
    /// calling `draw` yourself.
    ///
    /// Pass 0 to disable a previously set cap. By default, there isn't one.
    pub fn with_cap(mut self, max_fps: usize) -> Self {
        self.fps = match max_fps {
            0 => None,
            nz => Some((Duration::from_secs_f32(1.0 / nz as f32), Instant::now())),
        };
        self
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
        self.io
            .poll_input()
            .map(|o| o.map(|input| self.feed(root, input)))
    }

    /// Rerender the screen by passing an [`Action::Redraw`] into it.
    pub fn refresh<'s, A: Attachment<'s>>(&'s mut self, root: A) -> A::Output {
        self.feed(root, Action::Redraw)
    }

    /// Manually feed in an action, doing everything as though it was taken from the `IoSystem`.
    ///
    /// This is probably most obviously useful for `adapter.feed(&mut attachment, Action::Redraw)`. But that shouldn't
    /// be necessary if you're handling inputs immediately before drawing. Window resizes, etc. will send an
    /// [`Action::Redraw`] that'll trigger a rerender anyway.
    pub fn feed<'s, A: Attachment<'s>>(&'s mut self, root: A, input: Action) -> A::Output {
        self.current.resize(self.io.size());
        let region = Region::new(&mut self.current, input);
        region.attach(root)
    }

    /// Draw the stored screen to the display.
    ///
    /// This will **not** re-render anything if the screen isn't the right size -- it'll just try to draw. See the
    /// caveats in [`IoSystem::draw`] for more info.
    pub fn draw(&mut self) -> Result<()> {
        if self.current == self.old {
            return Ok(());
        }
        if let Some((delta, ref mut next_draw)) = self.fps {
            let now = Instant::now();
            if now < *next_draw {
                return Ok(());
            }
            *next_draw = now + delta;
        }
        self.io.draw(&self.current)?;
        // preserve the screen we just drew as the old one, start rendering to the old old one
        std::mem::swap(&mut self.old, &mut self.current);
        Ok(())
    }

    /// [Stop](IoSystem::stop) the `IoSystem`.
    pub fn stop(&mut self) {
        self.io.stop()
    }
}
