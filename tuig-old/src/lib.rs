mod agent;
mod game;
mod io;
mod runner;
mod timing;
mod util;

pub mod format;

// TODO: reorganize the library so these are more naturally exposed
pub use agent::{Agent, ControlFlow, WaitHandle};
pub use game::{Game, Message, Replies, Response};
pub use io::{
    helpers::{TextInput, TextInputRequest},
    input::{Action, Key},
    output::Screen,
    sys::{self, IoRunner, IoSystem},
    xy::XY,
};
#[cfg(feature = "__run")]
pub use runner::Runner;

/// Re-exports you may find useful when using tui-engine; use it like:
///
/// ```rs
/// use tuig::prelude::*;
/// ```
pub mod prelude {}
