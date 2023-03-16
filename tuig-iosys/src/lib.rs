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
//! [`IoRunner`] must be run on the main thread, to ensure things work as expected on Windows GUIs. Find systems in:
//! 
//! - [`gui`], for graphical IO. These all use `winit` but the pixel pushing backends vary.
//! - [`cli`], for terminal IO. The only thing they all have in common is writing to `stdout`.
//! - [`misc`], for, uh, well, miscellaneous IO.
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
//! their features but it'll yell at you.
//! 
//! There are also features controlling what parts of [`fmt`] are available. This doesn't influence the selection of
//! backends, but backends will cheerfully ignore anything they don't understand. See that module for details.

#![cfg_attr(not(feature = "std"), no_std)]

mod fmt;
mod gui;
mod cli;
mod misc;
