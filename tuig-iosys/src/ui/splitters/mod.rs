use super::Region;

pub mod cols;

/// Common trait implemented by all the things you can pass to [`Region::split`].
///
/// This trait isn't really meant to be implemented externally, but you can if you want.
pub trait Splitter<'r> {
    type Output;
    fn split(self, parent: Region<'r>) -> Self::Output;
}
