//! This crate was designed and built with [`tuig`](../tuig/index.html) in mind. If you're using tuig, see its docs,
//! too, for how it uses this crate.
//!
//! That said, `tuig-iosys` tries to be somewhat more broadly useful. It's still hyperfocused on textmode outputs, but
//! it doesn't have all the extra "game engine" stuff. If you want to use it yourself, there are two central parts to
//! familiarize yourself with.
//!
//! The first is [`Screen`]. It's a grid of formatted characters you can freely draw to. (If you'd like a UI system
//! designed for chargrids that renders straight to it, consider [`tuig-ui`](https://crates.io/crates/tuig-ui)!)
//!
//! The second is [`IoSystem`]. You can use `load!` to pick one based on available features and which one succeeds
//! first, or you can load them yourself. The `IoSystem` can be passed around wherever you like, but its associated
//! [`IoRunner`] must be run on the main thread, to ensure things work as expected on Windows GUIs. Builtin backends
//! are enabled by features and available in [`backends`].
//!
//! # Features
//!
//! There's one feature to enable each builtin backend; see each backend for details. (When you see "available on
//! has_backend only" in these docs, it refers to enabling at least one builtin.)
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
//! If you're making a GUI backend, though, consider implementing [`im::GuiRenderer`] and using [`im::GuiSystem`] and
//! [`im::GuiRunner`] -- it uses `winit` to generate a graphical context and will ensure your GUI system handles input
//! in the exact same way as every other.

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
mod xy;

pub use crate::{
    action::{Action, Key, MouseButton},
    error::{Error, Result},
    screen::Screen,
    traits::{IoRunner, IoSystem},
    xy::XY,
};

/// Helper types for implementing your own (primarily graphical) IO systems.
#[cfg(feature = "gui")]
pub mod im {
    pub use super::graphical::{GuiRenderer, GuiRunner, GuiSystem, BOLD_TTF, REGULAR_TTF};
}

/// Available rendering backends. See the [`IoSystem`] and [`IoRunner`] docs for more information.
pub mod backends {
    #[allow(unused)]
    use super::*;

    macro_rules! backends {
        ( $(
            $( #[ $( $attr:meta ),* ] )*
            $feat:literal, $name:ident => $sys:ty, $run:ty
        );* $(;)? ) => { paste::paste! { $(
            #[cfg(feature = $feat)]
            $( #[ $( $attr ),* ] )*
            pub type [< $name System >] = $sys;

            #[cfg(feature = $feat)]
            #[doc = concat!("[`IoRunner`] for [`", stringify!($name), "System`].")]
            pub type [< $name Runner >] = $run;
        )* } }
    }

    backends! {
        /// An [`IoSystem`] that does nothing at all.
        ///
        /// Rendering is a no-op, input never comes, and the runner just waits forever for `stop`. Meant primarily for
        /// testing, e.g. `mass-messages`.
        "nop", Nop => misc::nop::NopSystem, misc::nop::NopRunner;
        /// Crossterm-based CLI IO system.
        ///
        /// This uses an actual terminal to do its input/output. That means it should also work over SSH, Telnet, and
        /// the like. It'll probably also work without even having a desktop environment, in a bare VTTY.
        "cli_crossterm", Crossterm => terminal::crossterm::CtSystem, terminal::crossterm::CtRunner;
        /// Window-based IO system, with CPU rendering.
        ///
        /// Because this is backed by [`softbuffer`](https://crates.io/crates/softbuffer), it should be more or less
        /// as widely compatible as any graphical system can possibly be. And it should only get better with time and
        /// updates to the dependencies.
        "gui_softbuffer", Softbuffer => graphical::GuiSystem<graphical::softbuffer::SoftbufferBackend>, graphical::GuiRunner;
    }
}

type LoadError =
    core::result::Result<(Box<dyn IoSystem>, Box<dyn IoRunner>), BTreeMap<&'static str, Error>>;
/// Based on IO system features enabled, attempt to initialize an IO system, in the same manner as [`load!`].
///
/// This returns things boxed so they can be used as trait objects, which provides nicer ergonomics at the
/// cost of slightly lower max performance.
pub fn load() -> LoadError {
    #[allow(unused)]
    fn cb(
        sys: impl IoSystem + 'static,
        run: impl IoRunner + 'static,
    ) -> (Box<dyn IoSystem>, Box<dyn IoRunner>) {
        (Box::new(sys), Box::new(run))
    }
    load!(cb)
}

tuig_pm::make_load! {
    /// Based on IO system features enabled, attempt to initialize an IO system; in order:
    ///
    /// - NOP (`nop`), for benchmarks
    /// - Vulkan GUI (`gui_vulkan`)
    /// - OpenGL GUI (`gui_opengl`)
    /// - CPU-rendered GUI (`gui_softbuffer`)
    /// - crossterm CLI (`cli_crossterm`)
    ///
    /// This macro takes a function or method to call with the loaded `impl IoSystem`. That structure is weird but it
    /// enables having ownership of the varied types, without needing a `Box`. If you *want* a `Box`, you can just use
    /// [`load()`] instead.
    ///
    /// The callback can be any "function call", up to the parens, e.g. `run` or `self.start`. It will be called as
    /// `$thing(iosys, iorun)`. If it's called, this macro "returns" `Ok(())`. Otherwise, all attempted loads failed,
    /// and this macro "returns" `Err(map)`, where `map` maps `&'static str` feature name to `io::Error` failure.
    "nop" => $crate::backends::NopSystem::new(),
    "gui_softbuffer" => $crate::backends::SoftbufferSystem::new(20.0),
    "cli_crossterm" => $crate::backends::CrosstermSystem::new(),
}
