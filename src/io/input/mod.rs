use crate::io::XY;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Key {
    Char(char),
    F(u8),
    Escape,
    Backspace,
    Up, Down, Left, Right,
    PageUp, PageDown, Home, End,
    Delete, Insert,
    Tab, Enter,
    LeftShift, RightShift,
    LeftCtrl, RightCtrl,
    LeftAlt, RightAlt,
    LeftSuper, RightSuper,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
    ScrollUp,
    ScrollDown,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Action {
    /// A key was pressed. Note this theoretically handles modifiers by sending them when they're pressed, but
    /// depending on the input mechanism it may only be able to send them when a non-modifier key is pressed.
    KeyPress {
        key: Key,
    },
    /// A key was let go. Note this theoretically handles modifiers by sending them when they're let go, but
    /// depending on the input mechanism it may only be able to send them when a non-modifier key is pressed.
    KeyRelease {
        key: Key,
    },
    /// A mouse button was pressed at the given location.
    MousePress {
        button: MouseButton,
        pos: XY,
    },
    /// A mouse button was released at the given location.
    MouseRelease {
        button: MouseButton,
        pos: XY,
    },
    /// The mouse has moved to a new location, possibly while holding a button
    MouseMove {
        button: Option<MouseButton>,
        pos: XY,
    },
    /// Some unknown input was received, with a description of what it was
    Unknown(String),
    /// Trying to read input let to some kind of error, with a description
    Error(String),
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
