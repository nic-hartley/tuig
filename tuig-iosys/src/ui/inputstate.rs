use crate::{Action, Key};

/// Tracks and presents the current state of inputs, based on past ones.
///
/// You must manually feed this each input you receive. If you miss any, it might fall out of sync with reality,
/// though because events are absolute, there's no risk of the classic "exactly inverted state" desync.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct InputState {
    /// Whether either Shift key is currently being held
    pub shift: bool,
    /// Whether either Control key is currently being held
    pub ctrl: bool,
    /// Whether either Alt key is currently being held
    pub alt: bool,
    /// Whether either Super (Windows) key is currently being held
    // (`super` is a keyword)
    pub super_: bool,
}

impl InputState {
    /// Create a new key state tracker
    #[cfg_attr(coverage, no_coverage)]
    #[allow(unused)]
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

    /// Whether Ctrl, Alt, or Super are held, i.e. whether a hotkey is being used, vs. normal typing.
    ///
    /// This is mostly to clean up some code in text input, simplifying and sharing conditionals.
    pub fn hotkeying(&self) -> bool {
        self.ctrl || self.alt || self.super_
    }
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! testset {
        ( $( $side:ident $enum:ident => $field:ident, $hotkey:literal );* $(;)?) => { paste::paste! { $(
            #[test]
            #[allow(non_snake_case)]
            fn [< press_ $side _ $enum >]() {
                let mut ms = InputState::default();
                assert!(!ms.hotkeying(), "hotkeying on by default");
                assert!(ms.press(&Key::[<$side $enum>]), "key should be handled");
                assert!(ms.$field, "{} not set after key press", ms.$field);
                assert_eq!(ms.hotkeying(), $hotkey, "hotkeying in wrong state afterwards")
            }
            #[test]
            #[allow(non_snake_case)]
            fn [< release_ $side _ $enum >]() {
                let mut ms = InputState {
                    $field: true,
                    ..Default::default()
                };
                assert!(ms.release(&Key::[<$side $enum>]), "key should be handled");
                assert!(!ms.$field, "{} set after key release", ms.$field);
            }

            #[test]
            #[allow(non_snake_case)]
            fn [< press_ $side _ $enum _action >]() {
                let mut ms = InputState::default();
                assert!(!ms.hotkeying(), "hotkeying on by default");
                assert!(ms.action(&Action::KeyPress { key: Key::[<$side $enum>] }), "key should be handled");
                assert!(ms.$field, "{} not set after key press", ms.$field);
                assert_eq!(ms.hotkeying(), $hotkey, "hotkeying in wrong state afterwards")
            }
            #[test]
            #[allow(non_snake_case)]
            fn [< release_ $side _ $enum _action >]() {
                let mut ms = InputState {
                    $field: true,
                    ..Default::default()
                };
                assert!(ms.action(&Action::KeyRelease { key: Key::[<$side $enum>] }), "key should be handled");
                assert!(!ms.$field, "{} set after key release", ms.$field);
            }
        )* } };
        ( $($enum:ident => $field:ident, $hotkey:literal);* $(;)?) => { $(
            testset! {
                Left $enum => $field, $hotkey;
                Right $enum => $field, $hotkey;
            }
        )* };
    }

    testset! {
        Shift => shift, false;
        Ctrl => ctrl, true;
        Alt => alt, true;
        Super => super_, true;
    }

    macro_rules! testignored {
        ( $( $name:ident: $func:ident($( $arg:expr ),* $(,)?) ),* $(,)? ) => { $(
            #[test]
            fn $name() {
                let mut ms = InputState::default();
                assert!(!ms.$func($($arg),*), "unrelated input had an effect");
                assert_eq!(ms, Default::default());
            }
        )* }
    }
    testignored! {
        other_press_ignored: press(&Key::Char('f')),
        other_release_ignored: release(&Key::Char('f')),
        other_press_action_ignored: action(&Action::KeyPress { key: Key::Char('f') }),
        other_release_action_ignored: action(&Action::KeyRelease { key: Key::Char('f') }),
        other_action_ignored: action(&Action::Redraw),
    }
}
