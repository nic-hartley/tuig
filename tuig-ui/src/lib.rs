//! The text UI system built for [`tuig`](https://docs.rs/tuig), but usable outside of it.
//!
//! There are three main ways to use this crate.
//!
//! **When rendering a UI,** you'll probably use [`Adapter`] to start out, since it encapsulates most of the required
//! logic and just leaves you with the UI to write. You might find it doesn't meet your needs, though, in which case
//! its code can serve as a starting point. If there's some obvious shortfall, please open a feature request, or even
//! make your own crate with a more advanced one, since it's only hitting public APIs.
//!
//! **When writing a UI,** you'll get a [`Region`], possibly because you implemented [`Attachment`]. You can
//! [`split`](Region::split) it in a variety of ways -- with anything that implements [`Splitter`] -- and put more
//! [`Attachment`]s in the resulting child regions.
//!
//! **When making a totally new UI element,** you'll implement [`RawAttachment`]. But try to minimize how complex your
//! raw attachments are -- keeping them simple makes them easy to compose and adjust later.
//!
//! There are a couple of basic examples in the repo that you may find useful.

extern crate alloc;

mod adapter;
pub use adapter::Adapter;
pub mod attachments;
pub use attachments::{Attachment, RawAttachment};
mod bounds;
pub(crate) use bounds::Bounds;
mod inputstate;
pub use inputstate::InputState;
mod region;
pub use region::Region;
mod view;
pub use tuig_iosys::Screen;
pub use view::ScreenView;

#[doc(hidden)]
pub mod splitters;
pub use splitters::Splitter;
#[doc(hidden)]
pub mod macros;
