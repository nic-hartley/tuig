//! The IO system/backend traits themselves.

use crate::{Action, Result, Screen, XY};

/// An input/output system.
///
/// The output is called a "display" to distinguish it from the [`Screen`].
///
/// This object is meant to be associated with a [`IoRunner`], which will run infinitely on the main thread while this
/// is called from within the event system.
pub trait IoSystem: Send {
    /// Actually render a [`Screen`] to the display.
    ///
    /// Please don't `clone` the screen to send it to the `IoRunner` unless you *really really* have to. The reason
    /// this takes a reference is so allocations can be reused.
    fn draw(&mut self, screen: &Screen) -> Result<()>;
    /// Get the size of the display, in characters.
    ///
    /// This is the size of the actual renderable space. If there's extra room, e.g. a few pixels but not enough for a
    /// whole column of characters, it doesn't count. (And it's up to the IoSystem to decide what to do with it --
    /// usually, paint it a reasonable background color.)
    fn size(&self) -> XY;

    /// Wait for the next user input.
    fn input(&mut self) -> Result<Action>;
    /// If the next user input is available, return it. Otherwise, return `None`.
    ///
    /// Basically a non-blocking [`Self::input`].
    fn poll_input(&mut self) -> Result<Option<Action>>;

    /// Tells the associated [`IoRunner`] to stop and return control of the main thread, and tell the [`IoSystem`] to
    /// dispose of any resources it's handling.
    ///
    /// This **must not** wait for the runner to finish tearing down, to avoid deadlocks in the singlethreaded mode.
    ///
    /// This will always be the last method called on this object (unless you count `Drop::drop`) so feel free to
    /// panic in the others if they're called after this one, especially `draw`.
    fn stop(&mut self);
}

/// The other half of an [`IoSystem`].
///
/// This is used to do any processing that has to be done on the main thread.
pub trait IoRunner {
    /// Execute one 'step', which should be quick and must be non-blocking. Returns whether an exit has been requested
    /// (i.e. by [`IoSystem::stop`]) since the last time `step` was called.
    ///
    /// **Warning**: This function may cause issues, e.g. on graphical targets it might block while the window is
    /// being resized, [due to the underlying library][1]. Use it with caution, or only with backends you know work
    /// well with it.
    ///
    /// Will always be called on the main thread.
    ///
    ///  [1]: https://docs.rs/winit/latest/winit/platform/run_return/trait.EventLoopExtRunReturn.html#caveats
    #[must_use]
    fn step(&mut self) -> bool;

    /// Run until the paired [`IoSystem`] says to [stop](IoSystem::stop).
    ///
    /// Will always be called on the main thread.
    ///
    /// The default implementation just runs `while !self.step() { }`.
    fn run(&mut self) {
        while !self.step() {}
    }
}
