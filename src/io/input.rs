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
