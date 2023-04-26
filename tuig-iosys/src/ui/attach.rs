use crate::Action;

use super::{Region, ScreenView};

pub trait Attachment<'s> {
    type Output;
    fn attach(self, region: Region<'s>) -> Self::Output;
}

pub trait RawAttachment<'s> {
    type Output;
    fn raw_attach(self, input: Action, screen: ScreenView<'s>) -> Self::Output;
}

impl<'s, RAO, RA: RawAttachment<'s, Output = RAO>> Attachment<'s> for RA {
    type Output = RAO;
    fn attach(self, region: Region<'s>) -> Self::Output {
        let (input, screen) = region.raw_pieces();
        self.raw_attach(input, screen)
    }
}

impl<'s, T, F: FnOnce(Action, ScreenView<'s>) -> T> RawAttachment<'s> for F {
    type Output = T;
    fn raw_attach(self, input: Action, screen: ScreenView<'s>) -> Self::Output {
        self(input, screen)
    }
}
