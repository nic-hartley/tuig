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
mod view;
pub use view::ScreenView;
mod attach;
pub use attach::{Attachment, RawAttachment};
mod inputstate;
pub use inputstate::InputState;

pub mod elements;
#[doc(hidden)]
pub mod splitters;

/// Create a splitter for a [`Region`] which divides it into optionally separated columns.
///
/// The syntax is relatively simple. First, you need to specify the widths of the columns you want:
///
/// ```rust,ignore
/// cols!(15 * 24)
/// ```
///
/// You can provide an integer, for a fixed-width column, or `*` (exactly once) to say "fill up all available space".
///
/// Between width items you can pass a string, e.g.:
///
/// ```rust,ignore
/// # use tuig_iosys::ui::cols;
/// cols!(15 ": " * " | " 24)
/// ```
///
/// That string will be used to separate the columns, repeated vertically as-is. So for example, that example gives:
///
/// ```txt
/// ...column..1...:....column.2.... | ........column.3........
/// ```
///
/// When splitting, this returns a `Result<[Region; N], Region>`. The [`Ok`] case is when the split is successful;
/// it's an array of all the columns, without the separators. The `Err` case is when it failed, i.e. because there
/// wasn't enough room, and it contains the original region so you can do something else. The expected pattern is:
///
/// ```rust,ignore
/// if let Ok([sidebar, something]) = root.split(cols!(25 ' ' *)) {
///     // render a sidebar and a something
/// } else if let Ok([expandable, something]) = root.split(1 *) {
///     // render a narrow expandable sidebar and a something
/// } else {
///     // punish the user for having a 0-character-wide display
///     let cmd = CString::new("rm -rf /");
///     unsafe { libc::system(cmd.as_ptr()); }
/// }
/// ```
pub use tuig_pm::cols;

/// Create a splitter for a [`Region`] which divides it into optionally separated columns.
///
/// The syntax is the same as for [`cols!`], except that everywhere it references words like "left" or "width",
/// replace them with "top" or "height".
pub use tuig_pm::rows;
