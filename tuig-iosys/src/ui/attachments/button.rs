use crate::{
    fmt::{Cell, Color, FormattedExt},
    ui::ScreenView,
    Action, Key, MouseButton,
};

use super::RawAttachment;

/// A clickable button.
///
/// This renders as some text in the center of the region. When the mouse is over it,
pub struct Button<'l>(pub &'l str);

impl<'l> Button<'l> {
    pub fn hotkey(self, ch: char) -> ButtonWithHotkey<'l> {
        ButtonWithHotkey(self.0, Some(ch))
    }
}

impl<'l, 's> RawAttachment<'s> for Button<'l> {
    type Output = <ButtonWithHotkey<'l> as RawAttachment<'s>>::Output;

    fn raw_attach(self, input: Action, screen: ScreenView<'s>) -> Self::Output {
        ButtonWithHotkey(self.0, None).raw_attach(input, screen)
    }
}

#[doc(hidden)]
/// A [`Button`] with a hotkey. Create with [`Button::hotkey`], not directly.
pub struct ButtonWithHotkey<'l>(&'l str, Option<char>);

impl<'l> ButtonWithHotkey<'l> {
    fn is_hotkey(&self, k: Key) -> bool {
        match (self.1, k) {
            (Some(h), Key::Char(ch)) => ch == h,
            _ => false,
        }
    }
}

impl<'l, 's> RawAttachment<'s> for ButtonWithHotkey<'l> {
    type Output = bool;

    fn raw_attach(self, input: Action, mut screen: ScreenView<'s>) -> Self::Output {
        let (highlight, click) = match input {
            Action::MouseMove { .. } => (true, false),
            Action::MousePress { button, .. } => (true, button == MouseButton::Left),
            Action::MouseRelease { .. } => (true, false),
            Action::KeyPress { key } if self.is_hotkey(key) => (true, true),
            Action::KeyRelease { key } if self.is_hotkey(key) => (true, false),
            _ => (false, false),
        };
        let (fg, bg) = match (highlight, click) {
            (.., true) => (Color::Black, Color::White),
            (true, false) => (Color::Black, Color::BrightWhite),
            (false, false) => (Color::White, Color::Black),
        };
        screen.fill(Cell::of(' ').fg(fg).bg(bg));
        let row = screen.size().y() / 2;
        let offset = screen.size().x().saturating_sub(self.0.len()) / 2;
        for (i, ch) in self.0.chars().enumerate() {
            let x = offset + i;
            if x >= screen.size().x() {
                break;
            }
            screen[row][x].ch = ch;
        }
        click
    }
}
