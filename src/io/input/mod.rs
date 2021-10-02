use tokio::sync::broadcast;

use crate::io::XY;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Key {
    Char(u8),
    Delete,
    Backspace,
    F(u8),
    Up, Down, Left, Right,
    Enter,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Action {
    KeyPress {
        key: Key,
        shift: bool,
        ctrl: bool,
        alt: bool,
    },
    MousePress {
        button: u8,
        pos: XY,
        click: bool,
    },
    Unknown(Vec<u8>),
}

/// Common interface for all sources of input.
///
/// It's a bit indirect: rather than just returning the next Action, it's used to set up a queue. That queue is what's
/// actually read from, because tokio provides some guarantees about it that it doesn't provide about general futures.
pub trait Input {
    /// Attaches a listener. This is a Tokio queue rather than just an iterable so it can be asynchronously waited on
    /// without losing any data. Note that this queue closing **must not** close the actual input stream; that should
    /// only happen when the `impl Input` is dropped.
    fn listen(&mut self) -> broadcast::Receiver<Action>;
}

impl dyn Input + '_ {
    pub fn get() -> Box<dyn Input> {
        if cfg!(feature = "force_in_test") {
            return Box::new(test::UntimedStream::of(&[]));
        }

        unimplemented!("other screen types");
    }
}

pub mod test;
