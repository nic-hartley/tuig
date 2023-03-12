//! This module provides input adapters for each of the various I/O mechanisms supported. Each one is controlled by
//! a feature named similarly and exports a struct implementing `IoSystem`. The actual intended input and output APIs
//! are in the `input` and `output` modules.

use std::io;

use super::{input::Action, output::Screen, xy::XY};

#[cfg(feature = "sys_cli")]
pub mod ansi_cli;

#[cfg(feature = "__sys_gui")]
pub mod gui;

#[cfg(feature = "sys_nop")]
pub mod nop;

/// An input/output system.
///
/// The output is called a "display" to distinguish it from the [`Screen`].
///
/// This object is meant to be associated with a [`IoRunner`], which will run infinitely on the main thread while this
/// is called from within the event system.
pub trait IoSystem: Send {
    /// Actually render a [`Screen`] to the display.
    fn draw(&mut self, screen: &Screen) -> io::Result<()>;
    /// Get the size of the display, in characters.
    fn size(&self) -> XY;

    /// Wait for the next user input.
    fn input(&mut self) -> io::Result<Action>;
    /// If the next user input is available, return it.
    fn poll_input(&mut self) -> io::Result<Option<Action>>;

    /// Tells the associated [`IoRunner`] to stop and return control of the main thread, and tell the [`IoSystem`] to
    /// dispose of any resources it's handling.
    ///
    /// This **must** return even if the `IoRunner` isn't done tearing down, to avoid deadlocks in the singlethreaded
    /// mode.
    ///
    /// This will always be the last method called on this object (unless you count `Drop::drop`) so feel free to
    /// panic in the others if they're called after this one, especially `draw`.
    fn stop(&mut self);
}

/// The other half of an [`IoSystem`].
///
/// This type exists so that things which need to run on the main thread specifically, can.
pub trait IoRunner {
    /// Execute one 'step', which should be quick and must be non-blocking. Returns whether an exit has been requested
    /// (i.e. by [`IoSystem::stop`]) since the last time `step` was called.
    ///
    /// Will always be called on the main thread.
    #[must_use]
    fn step(&mut self) -> bool;

    /// Run until the paired [`IoSystem`] tells you to stop.
    ///
    /// Will always be called on the main thread.
    ///
    /// The default implementation just runs `while !self.step() { }`.
    fn run(&mut self) {
        while !self.step() {}
    }
}

/// Based on IO system features enabled, attempt to initialize an IO system; in order:
///
/// - NOP (`nop`), for benchmarks
/// - Vulkan GUI (`gui_vulkan`)
/// - OpenGL GUI (`gui_opengl`)
/// - CPU-rendered GUI (`gui_cpu`)
/// - crossterm CLI (`cli_crossterm`)
///
/// This macro takes a function or method to call with the loaded `impl IoSystem`. That structure is weird but it
/// enables having ownership of the varied types, without needing a `Box`.
///
/// The callback can be any "function call", up to the parens, e.g. `run` or `self.start`. It will be called as
/// `$thing(iosys, iorun)`. If it's called, this macro "returns" `Ok(())`. Otherwise, all attempted loads failed, and
/// this macro "returns" `Err(map)`, where `map` maps `&'static str` feature name to `io::Error` failure.
#[cfg(feature = "__sys")]
#[macro_export]
macro_rules! load {
    ( @@one $errs:ident { $( [ $( $callback:tt )* ] $feature:literal => $init:expr );* $(;)? } ) => { $(
        #[cfg(feature = $feature)] {
            match $init {
                Ok((iosys, iorun)) => {
                    break Ok($( $callback )* (iosys, iorun));
                }
                Err(e) => {
                    $errs.insert($feature, e);
                }
            }
        }
    )* };
    ( $( $callback:tt )* ) => { loop {
        use $crate::io::sys::*;
        let mut errs = std::collections::HashMap::new();
        $crate::io::sys::load! { @@one errs {
            [ $( $callback )* ] "sys_nop" => nop::NopSystem::new();
            [ $( $callback )* ] "sys_gui_softbuffer" => gui::Gui::<gui::softbuffer::SoftbufferBackend>::new(20.0);
            [ $( $callback )* ] "sys_cli" => ansi_cli::AnsiIo::get();
        } }
        break Err(errs);
    } };
}

#[cfg(feature = "__sys")]
pub use load;
