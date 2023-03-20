//! This crate was designed and built with [`tuig`](https://crates.io/crates/tuig) in mind. Most of the time you use
//! this, you'll probably use it indirectly, by implementing `tuig::Game` and using a `tuig::Runner`.
//! 
//! That said, `tuig-iosys` tries to be somewhat more broadly useful. If you want to use it yourself, there are two
//! central parts to familiarize yourself with.
//! 
//! The first is [`Screen`]. It's a grid of formatted characters you can freely draw to. With the `ui` feature, it
//! also has a small, modular UI system; see [`ui`] for more information. The formatting is in [`fmt`]; see that for
//! more information.
//! 
//! The second is [`IoSystem`]. You can use `load!` to pick one based on available features and which one succeeds
//! first, or you can load them yourself. The `IoSystem` can be passed around wherever you like, but its associated
//! [`IoRunner`] must be run on the main thread, to ensure things work as expected on Windows GUIs. Builtin backends
//! are enabled by features and available in [`backends`].
//! 
//! If you want to implement your own `tuig-iosys` compatible renderer, you should implement the `IoBackend` trait.
//! As a library user, you shouldn't actually use it, but it will ensure you have all the expected functions, named
//! the expected things with the expected function signatures.
//! 
//! # Features
//! 
//! There's one feature to enable each builtin backend; see each backend for details.
//! 
//! The `std` feature, on by default, enables `std`. Some backends aren't available without it; you can still turn on
//! their features but it'll yell at you. All of `fmt` is `no_std` compatible.
//! 
//! There are also features controlling what extensions to `fmt` are available. This doesn't influence the selection of
//! backends, but backends will cheerfully ignore anything they don't understand. See that module for details.

#![cfg_attr(not(feature = "std"), no_std)]

use alloc::borrow::Cow;

/// Re-exported for the [`load!`] macro.
pub use alloc::{collections::BTreeMap, boxed::Box, string::String};

extern crate alloc;

mod graphical;
mod terminal;
mod misc;

pub mod fmt;
mod screen;
mod action;
mod xy;
#[cfg(feature = "ui")]
mod ui;

mod util;

#[non_exhaustive]
#[derive(Debug)]
pub enum Error {
    /// An `io::Error` occurred.
    #[cfg(feature = "std")]
    Io(std::io::Error),
    /// While a [`graphical`] backend was initializing, `winit` errored out.
    #[cfg(feature = "gui")]
    Winit(winit::error::ExternalError),
    /// Just directly contains an error message.
    Bare(Cow<'static, str>),
}

#[cfg(feature = "std")]
impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

#[cfg(feature = "gui")]
impl From<winit::error::ExternalError> for Error {
    fn from(value: winit::error::ExternalError) -> Self {
        Self::Winit(value)
    }
}

impl From<&'static str> for Error {
    fn from(value: &'static str) -> Self {
        Self::Bare(Cow::Borrowed(value))
    }
}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Self::Bare(Cow::Owned(value))
    }
}

type Result<T> = core::result::Result<T, Error>;

/// An input/output system.
///
/// The output is called a "display" to distinguish it from the [`Screen`].
///
/// This object is meant to be associated with a [`IoRunner`], which will run infinitely on the main thread while this
/// is called from within the event system.
pub trait IoSystem: Send {
    /// Actually render a [`Screen`] to the display.
    fn draw(&mut self, screen: &screen::Screen) -> Result<()>;
    /// Get the size of the display, in characters.
    fn size(&self) -> xy::XY;

    /// Wait for the next user input.
    fn input(&mut self) -> Result<action::Action>;
    /// If the next user input is available, return it.
    fn poll_input(&mut self) -> Result<Option<action::Action>>;

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

pub use crate::{
    screen::Screen,
    xy::XY,
    action::Action,
};

/// Available rendering backends. See the [`IoSystem`] and [`IoRunner`] docs for more information.
pub mod backends {
    #[allow(unused)]
    use super::*;

    #[cfg(feature = "nop")]
    pub type NopSystem = misc::nop::NopSystem;
    #[cfg(feature = "nop")]
    pub type NopRunner = misc::nop::NopRunner;

    #[cfg(feature = "cli_crossterm")]
    pub type CrosstermSystem = terminal::crossterm::CtSystem;
    #[cfg(feature = "cli_crossterm")]
    pub type CrosstermRunner = terminal::crossterm::CtRunner;

    #[cfg(feature = "gui_softbuffer")]
    pub type SoftbufferSystem = graphical::GuiSystem<graphical::softbuffer::SoftbufferBackend>;
    #[cfg(feature = "gui_softbuffer")]
    pub type SoftbufferRunner = graphical::GuiRunner;
}

tuig_pm::make_load! {
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
    "nop" => $crate::backends::NopSystem::new(),
    "gui_softbuffer" => $crate::backends::SoftbufferSystem::new(20.0),
    "cli_crossterm" => $crate::backends::CrosstermSystem::new(),
}

/// Based on IO system features enabled, attempt to initialize an IO system, in the same manner as [`load!`].
/// 
/// This returns things boxed so they can be used as trait objects, which provides better ergonomics at the cost of
/// slightly lower max performance.
pub fn load() -> core::result::Result<(Box<dyn IoSystem>, Box<dyn IoRunner>), BTreeMap<&'static str, Error>> {
    #[allow(unused)]
    fn cb(sys: impl IoSystem + 'static, run: impl IoRunner + 'static) -> (Box<dyn IoSystem>, Box<dyn IoRunner>) {
        (Box::new(sys), Box::new(run))
    }
    load!(cb)
}
