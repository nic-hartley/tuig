use crate::io::input::{Action, Key};

/// Tracks and presents the current state of the keyboard's modifiers (Shift/Ctrl/Alt/Super).
#[derive(Default, Clone)]
pub struct ModState {
    /// Whether either Shift key is currently being held
    pub shift: bool,
    /// Whether either Control key is currently being held
    pub ctrl: bool,
    /// Whether either Alt key is currently being held
    pub alt: bool,
    /// Whether either Super key is currently being held (`super` is a keyword)
    pub super_: bool,
}

impl ModState {
    /// Create a new key state tracker
    pub fn new() -> Self {
        Default::default()
    }

    /// Handle a key press, returning whether this was affected (i.e. whether it was a modifier being pressed)
    pub fn press(&mut self, key: &Key) -> bool {
        match key {
            Key::LeftShift | Key::RightShift => self.shift = true,
            Key::LeftCtrl | Key::RightCtrl => self.ctrl = true,
            Key::LeftAlt | Key::RightAlt => self.alt = true,
            Key::LeftSuper | Key::RightSuper => self.super_ = true,
            _ => return false,
        }
        true
    }

    /// Handle a key release, returning whether this was affected (i.e. whether it was a modifier being released)
    pub fn release(&mut self, key: &Key) -> bool {
        match key {
            Key::LeftShift | Key::RightShift => self.shift = false,
            Key::LeftCtrl | Key::RightCtrl => self.ctrl = false,
            Key::LeftAlt | Key::RightAlt => self.alt = false,
            Key::LeftSuper | Key::RightSuper => self.super_ = false,
            _ => return false,
        }
        true
    }

    /// Handle an action, returning whether this was affected (i.e. whether it was a modifier being touched)
    pub fn action(&mut self, action: &Action) -> bool {
        match action {
            Action::KeyPress { key } => self.press(key),
            Action::KeyRelease { key } => self.release(key),
            _ => false,
        }
    }

    /// Whether Ctrl, Alt, or Super are held. Meant to be used to check whether the input should be taken as normal
    /// typing or a hotkey.
    pub fn hotkeying(&self) -> bool {
        self.ctrl || self.alt || self.super_
    }
}
