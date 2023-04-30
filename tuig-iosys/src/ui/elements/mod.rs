//! Builtin UI elements.
//!
//! Many of these also have dedicated convenience methods on [`Region`][super::Region], which are generally preferred
//! to using the types directly. That said, all those convenience methods do is call `Region::attach` on an object in
//! this module.

mod button;
mod scrollable;
mod textbox;
pub use textbox::{Textbox, TextboxData};
mod text_input;
