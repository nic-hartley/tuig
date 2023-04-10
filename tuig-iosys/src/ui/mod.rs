//! The text UI system.
//!
//! It's based on the idea of splitting regions and filling them. You start with a [`Region`] occupying the whole
//! screen, [`.split`](Region::split) it however you want, and then [`.attach`](Region::attach) something to the
//! subregions. It's an immediate mode API, meaning you don't build and maintain persistent UI trees. You rebuild it
//! every time with simple, direct method calls, imperatively divvying up and assigning the screen space.
//!
//! That assigned screen space is used for both input and output. Driving this system can be challenging, but at its
//! most basic level will look something like:
//!
//! TODO: figure out what this looks like by implementing it in tuig
//!
//! You can implement your own custom attachments through the [`Attachment`] trait. The simplest ways are through the
//! existing impls for various function types, but you might find custom impls useful for higher-level abstractions.
//! Attachments take a `Region` and can do anything with it that you can do directly, including attach more things.

#![cfg(feature = "ui")]

pub mod helpers;
pub mod widgets;

mod bounds;
pub(crate) use bounds::Bounds;
mod region;
pub use region::Region;
