use super::Region;

pub mod statics;

/// Common trait implemented by all the things you can pass to [`Region::split`].
///
/// This trait isn't really meant to be implemented externally, but you can if you want. If you really need custom
/// splitting behavior, you might have some trouble with useful methods on `Region` being private -- consider dropping
/// an issue/PR with suggestions to improve the API, as it's currently not very good.
pub trait Splitter<'r> {
    type Output;
    fn split(self, parent: Region<'r>) -> Self::Output;
}
