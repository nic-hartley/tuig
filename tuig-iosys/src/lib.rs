//! > *This crate was designed and built with [`tuig`](https://crates.io/crates/tuig) in mind. If you're using tuig,
//! see its documentation for information.*
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
//! # Features
//!
//! There's one feature to enable each builtin backend; see each backend for details.
//!
//! The `std` feature, on by default, enables `std`. Some backends aren't available without it; you can still turn on
//! their features but it'll yell at you. All of `fmt` is `no_std` compatible.
//!
//! There are also features controlling what extensions to `fmt` are available. This doesn't influence the selection of
//! backends, but backends will cheerfully ignore anything they don't understand. See that module for details.
//!
//! # Custom backends
//!
//! If you want to implement your own `tuig-iosys` compatible renderer, you'll need an implementation of each of those
//! traits. `IoSystem` is the `Send`/`Sync` handle with which to do the IO, and `IoRunner` occupies the main thread in
//! case you need that, e.g. for GUI targets.
//!
//! If you're implementing a GUI backend, though, consider implementing [`GuiBackend`] and using [`GuiSystem`] and
//! [`GuiRunner`] -- it uses `winit` to generate a graphical context and will ensure your GUI system handles input in
//! the exact same way as every other.

#![cfg_attr(doc, feature(doc_auto_cfg))]
#![cfg_attr(not(feature = "std"), no_std)]

/// Re-exported for the [`load!`] macro.
#[doc(hidden)]
pub use alloc::{boxed::Box, collections::BTreeMap, string::String};

extern crate alloc;

mod error;
mod traits;

mod graphical;
mod misc;
mod terminal;

mod action;
pub mod fmt;
mod screen;
#[cfg(feature = "ui")]
pub mod ui;
mod xy;

mod util;

pub use crate::{
    action::{Action, Key, MouseButton},
    error::{Error, Result},
    screen::Screen,
    traits::{IoRunner, IoSystem},
    xy::XY,
};
#[cfg(feature = "gui")]
pub use graphical::{GuiBackend, GuiRunner, GuiSystem};

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
    /// `$thing(iosys, iorun)`. If it's called, this macro "returns" `Ok(())`. Otherwise, all attempted loads failed,
    /// and this macro "returns" `Err(map)`, where `map` maps `&'static str` feature name to `io::Error` failure.
    "nop" => $crate::backends::NopSystem::new(),
    "gui_softbuffer" => $crate::backends::SoftbufferSystem::new(20.0),
    "cli_crossterm" => $crate::backends::CrosstermSystem::new(),
}
