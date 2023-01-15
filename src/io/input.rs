use crate::io::XY;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Key {
    Char(char),
    F(usize),
    Escape,
    Backspace,
    Up,
    Down,
    Left,
    Right,
    PageUp,
    PageDown,
    Home,
    End,
    Delete,
    Insert,
    Tab,
    Enter,
    LeftShift,
    RightShift,
    LeftCtrl,
    RightCtrl,
    LeftAlt,
    RightAlt,
    LeftSuper,
    RightSuper,
}

impl Key {
    pub fn is_shift(&self) -> bool {
        matches!(self, Self::LeftShift | Self::RightShift)
    }
    pub fn is_ctrl(&self) -> bool {
        matches!(self, Self::LeftCtrl | Self::RightCtrl)
    }
    pub fn is_alt(&self) -> bool {
        matches!(self, Self::LeftAlt | Self::RightAlt)
    }
    pub fn is_super(&self) -> bool {
        matches!(self, Self::LeftSuper | Self::RightSuper)
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
    ScrollUp,
    ScrollDown,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Action {
    /// A key was pressed. Note this theoretically handles modifiers by sending them when they're pressed, but
    /// depending on the input mechanism it may only be able to send them when a non-modifier key is pressed.
    KeyPress { key: Key },
    /// A key was let go. Note this theoretically handles modifiers by sending them when they're let go, but
    /// depending on the input mechanism it may only be able to send them when a non-modifier key is pressed.
    KeyRelease { key: Key },
    /// A mouse button was pressed at the given location.
    MousePress { button: MouseButton },
    /// A mouse button was released at the given location.
    MouseRelease { button: MouseButton },
    /// The mouse has moved to a new location, possibly while holding a button
    MouseMove { pos: XY },
    /// Allows pushing redraw notifications, rather than having to update constantly and risk missing it
    Resized,
    /// User requested the program end externally, e.g. clicking the X button in the window
    Closed,
    /// User requested that the program pause temporarily
    Paused,
    /// User, having requested that the program pause temporarily, has since requested that it unpause
    ///
    /// This may be fired spuriously, i.e. without an associated [`Paused`][Self::Paused]. These must be ignored.
    Unpaused,
    /// Some unknown input was received, with a description of what it was
    Unknown(String),
    /// Trying to read input let to some kind of error, with a description
    Error(String),
}
