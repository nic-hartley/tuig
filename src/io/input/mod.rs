use crate::io::XY;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Key {
    Char(u8),
    Delete,
    Backspace,
    F(u8),
    Up, Down, Left, Right,
    Enter,
    LeftCtrl, RightCtrl,
    LeftAlt, RightAlt,
    LeftSuper, RightSuper,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Action {
    /// A key was pressed. Note this theoretically handles modifiers by sending them when they're pressed, but
    /// depending on the input mechanism it may only be able to send them when a non-modifier key is pressed.
    KeyPress {
        key: Key,
    },
    /// A mouse button was clicked at the given location.
    MousePress {
        button: u8,
        pos: XY,
    },
    /// The mouse has moved to a new location.
    MouseMove {
        pos: XY,
    },
    /// Some unknown input was received, with a description of what it was
    Unknown(String),
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

    /// Discard any queued but unreceived inputs. Should return immediately. If an input comes in while this function
    /// is executing, it might be discarded or not, depending on the implementation.
    fn flush(&mut self);
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
pub mod ansi_cli;
