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
/// It's expected this will spawn another task to actually watch the input, then push that into a [`mpsc::channel`]
/// to eventually be fed out form [`next`].
#[async_trait::async_trait]
pub trait Input {
    /// Get the next input event.
    /// 
    /// This **must** be cancel-safe. If the returned future is cancelled before data is polled out, no inputs may
    /// be lost.
    async fn next(&mut self) -> Action;

    /// Discard any queued but unreceived inputs.
    async fn flush(&mut self);
}

impl dyn Input + '_ {
    pub fn get() -> Box<dyn Input> {
        if cfg!(feature = "force_in_blank") {
            return Box::new(test::UntimedStream::of(&[]));
        }
        return Box::new(test::UntimedStream::of(&[]));
    }
}

pub mod test;
