mod game;
mod agent;
mod io;
mod util;
mod runner;
mod timing;

pub mod format;

// TODO: reorganize the library so these are more naturally exposed
pub use game::{Game, Message, Replies, Response};
pub use runner::Runner;
pub use agent::{Agent, ControlFlow, WaitHandle};
pub use io::{input::{Action, Key}, output::Screen, helpers::{TextInput, TextInputRequest}, sys::{self, IoSystem, IoRunner}, xy::XY};

/// Re-exports you may find useful when using tui-engine; use it like:
/// 
/// ```rs
/// use tuig::prelude::*;
/// ```
pub mod prelude {

}