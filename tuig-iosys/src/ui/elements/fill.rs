use crate::{fmt::Cell, ui::{RawAttachment, ScreenView}, Action};

pub struct Fill(pub Cell);

impl<'s> RawAttachment<'s> for Fill {
    type Output = ();
    fn raw_attach(self, _: Action, mut screen: ScreenView<'s>) -> Self::Output {
        screen.fill(self.0)
    }
}
