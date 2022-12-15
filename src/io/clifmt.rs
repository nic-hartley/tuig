//! This submodule implements the CLI formatting system used by all the IO systems. It does that as several pieces:
//!
//! - [`Format`], which contains various common formatting options (i.e. the common ANSI ones)
//! - [`Text`] and [`Cell`], which apply a `Format` to anything `impl AsRef<str>`, and `char` respectively
//!     - They use a common subtype `Formatted` and `Deref` internally to ensure a common set of methods
//! - [`text!`], which constructs a `Text` or `Cell` based on the parameter
//!
//! `Text` and `Cell` are then used in the various UI widgets.

use rand::{distributions::Standard, prelude::Distribution, seq::SliceRandom, Rng};

/// The color of a piece of formatted text. Meant to be used through `Text` / `text!`. The numeric values are the ANSI
/// color codes for each color; that's also where the actual colors are from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Color {
    Black = 0,
    Red = 1,
    Green = 2,
    Yellow = 3,
    Blue = 4,
    Magenta = 5,
    Cyan = 6,
    White = 7,
    Default = 9,
    BrightBlack = 60,
    BrightRed = 61,
    BrightGreen = 62,
    BrightYellow = 63,
    BrightBlue = 64,
    BrightMagenta = 65,
    BrightCyan = 66,
    BrightWhite = 67,
}

impl Color {
    /// All of the colors supported, not including [`Color::Default`] (because that's to reset, not a real color
    pub fn all() -> [Color; 16] {
        [
            Color::Black,
            Color::BrightBlack,
            Color::Red,
            Color::BrightRed,
            Color::Green,
            Color::BrightGreen,
            Color::Yellow,
            Color::BrightYellow,
            Color::Blue,
            Color::BrightBlue,
            Color::Magenta,
            Color::BrightMagenta,
            Color::Cyan,
            Color::BrightCyan,
            Color::White,
            Color::BrightWhite,
        ]
    }

    pub fn count() -> usize {
        Self::all().len()
    }

    /// The name of the color as a string
    pub fn name(&self) -> &'static str {
        match self {
            Color::Black => "black",
            Color::BrightBlack => "bright black",
            Color::Red => "red",
            Color::BrightRed => "bright red",
            Color::Green => "green",
            Color::BrightGreen => "bright green",
            Color::Yellow => "yellow",
            Color::BrightYellow => "bright yellow",
            Color::Blue => "blue",
            Color::BrightBlue => "bright blue",
            Color::Magenta => "magenta",
            Color::BrightMagenta => "bright magenta",
            Color::Cyan => "cyan",
            Color::BrightCyan => "bright cyan",
            Color::White => "white",
            Color::BrightWhite => "bright white",
            Color::Default => "default",
        }
    }
}

// allow random color choice
impl Distribution<Color> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Color {
        *Color::all().choose(rng).unwrap()
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::Default
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Format {
    pub fg: Color,
    pub bg: Color,
    pub bold: bool,
    pub underline: bool,
    pub invert: bool,
}

impl Format {
    pub const NONE: Self = Format {
        fg: Color::Default,
        bg: Color::Default,
        bold: false,
        underline: false,
        invert: false,
    };
}

macro_rules! fmt_fn {
    ( $(
        $name:ident
        $(( $( $arg:ident: $type:ty ),* $(,)? ))?
        =>
        $field:ident $(.$subfield:ident)*
        = $val:expr
    ),* $(,)? ) => { $(
        #[must_use]
        fn $name(mut self $($(, $arg: $type )*)? ) -> Self {
            self.get_fmt_mut().$field $(.$subfield)* = $val;
            self
        }
    )* };
}

pub trait Formatted {
    fn get_fmt(&self) -> &Format;
    fn get_fmt_mut(&mut self) -> &mut Format;
}

pub trait FormattedExt: Formatted + Sized {
    #[must_use]
    fn fmt(mut self, fmt: Format) -> Self {
        *self.get_fmt_mut() = fmt;
        self
    }
    #[must_use]
    fn fmt_of(mut self, rhs: &dyn Formatted) -> Self {
        *self.get_fmt_mut() = rhs.get_fmt().clone();
        self
    }
    fmt_fn! {
        fg(c: Color) => fg = c,                         bg(c: Color) => bg = c,
        black => fg = Color::Black,                     on_black => bg = Color::Black,
        bright_black => fg = Color::BrightBlack,        on_bright_black => bg = Color::BrightBlack,
        red => fg = Color::Red,                         on_red => bg = Color::Red,
        bright_red => fg = Color::BrightRed,            on_bright_red => bg = Color::BrightRed,
        green => fg = Color::Green,                     on_green => bg = Color::Green,
        bright_green => fg = Color::BrightGreen,        on_bright_green => bg = Color::BrightGreen,
        yellow => fg = Color::Yellow,                   on_yellow => bg = Color::Yellow,
        bright_yellow => fg = Color::BrightYellow,      on_bright_yellow => bg = Color::BrightYellow,
        blue => fg = Color::Blue,                       on_blue => bg = Color::Blue,
        bright_blue => fg = Color::BrightBlue,          on_bright_blue => bg = Color::BrightBlue,
        magenta => fg = Color::Magenta,                 on_magenta => bg = Color::Magenta,
        bright_magenta => fg = Color::BrightMagenta,    on_bright_magenta => bg = Color::BrightMagenta,
        cyan => fg = Color::Cyan,                       on_cyan => bg = Color::Cyan,
        bright_cyan => fg = Color::BrightCyan,          on_bright_cyan => bg = Color::BrightCyan,
        white => fg = Color::White,                     on_white => bg = Color::White,
        bright_white => fg = Color::BrightWhite,        on_bright_white => bg = Color::BrightWhite,
        default => fg = Color::Default,                 on_default => bg = Color::Default,
        underline => underline = true,
        bold => bold = true,
        invert => invert = true,
    }
}

impl<F: Formatted> FormattedExt for F {}

macro_rules! fmt_type {
    (
        $( #[$($attr:meta),* $(,)?] )*
        $svis:vis struct $name:ident { $( $fvis:vis $field:ident: $type:ty ),* $(,)? }
    ) => {
        $( #[$($attr),*] )*
        $svis struct $name {
            $( $fvis $field: $type, )*
            _fmt: $crate::io::clifmt::Format,
        }
        impl $crate::io::clifmt::Formatted for $name {
            fn get_fmt(&self) -> &$crate::io::clifmt::Format {
                &self._fmt
            }
            fn get_fmt_mut(&mut self) -> &mut $crate::io::clifmt::Format {
                &mut self._fmt
            }
        }
        impl $name {
            pub const fn of( $($field: $type),* ) -> Self {
                Self {
                    $( $field, )*
                    _fmt: $crate::io::clifmt::Format::NONE,
                }
            }
            pub fn data(mut self $(, $field: $type)*) -> Self {
                $(
                    self.$field = $field;
                )*
                self
            }
        }
    };
}

fmt_type!(
    /// A single bit of formatted text. Note this isn't really meant to be used on its own, though it can be; the API
    /// is designed to be used through `text!`. To discourage direct use, the internals aren't documented.
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct Text {
        pub text: String,
    }
);

impl Text {
    pub fn plain(s: &str) -> Text {
        Text::of(s.into())
    }

    pub(super) fn with_text(&self, new_text: String) -> Text {
        let mut res = self.clone();
        res.text = new_text.into();
        res
    }
}

#[macro_export]
macro_rules! text1 {
    (
        $( $name:ident )*
        $text:literal
        $( ( $( $arg:expr ),* $(,)? ) )?
    ) => {
        Text::of(
            format!( $text $(, $( $arg ),* )? )
        ) $( . $name () )*
    };
}

#[macro_export]
macro_rules! text {
    ( $(
        $( $name:ident )*
        $text:literal
        $( ( $( $arg:expr ),* $(,)? ) )?
    ),+ $(,)? ) => {
        {
            #[allow(unused_imports)]
            use $crate::io::clifmt::{FormattedExt as _};
            vec![
                $(
                    $crate::io::clifmt::Text::of(
                        format!( $text $(, $( $arg ),* )? )
                    ) $( . $name () )*
                ),+
            ]
        }
    };
}

fmt_type! {
    /// A single character that's been formatted. This is really only meant to be used in `Screen`.
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct Cell { pub ch: char }
}

#[macro_export]
macro_rules! cell {
    ( $( $name:ident )* $( $char:literal )? ) => {
        {
            #[allow(unused_imports)]
            use $crate::io::clifmt::{FormattedExt as _};
            Cell::of($($char)?) $( .$name() )*
        }
    };
}

impl Cell {
    pub const BLANK: Cell = cell!(' ');
}
