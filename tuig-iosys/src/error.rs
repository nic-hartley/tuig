use alloc::borrow::Cow;

#[non_exhaustive]
#[derive(Debug)]
pub enum Error {
    /// An `io::Error` occurred.
    #[cfg(feature = "std")]
    Io(std::io::Error),
    /// While a [`graphical`] backend was initializing, `winit` errored out.
    #[cfg(feature = "gui")]
    Winit(winit::error::ExternalError),
    /// Just directly contains an error message.
    Bare(Cow<'static, str>),
}

#[cfg(feature = "std")]
impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

#[cfg(feature = "gui")]
impl From<winit::error::ExternalError> for Error {
    fn from(value: winit::error::ExternalError) -> Self {
        Self::Winit(value)
    }
}

impl From<&'static str> for Error {
    fn from(value: &'static str) -> Self {
        Self::Bare(Cow::Borrowed(value))
    }
}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Self::Bare(Cow::Owned(value))
    }
}

pub type Result<T> = core::result::Result<T, Error>;
