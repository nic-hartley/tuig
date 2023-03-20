//! Implements CLI-compatible text formatting.
//!
//! This module defaults to a 'lowest common subset' across all render targets. Additional functionality is enabled if
//! explicitly turned on, but be aware that incompatible render hardware will simply ignore it. To achieve "good" UI
//! across a variety of render targets, you'll need to write your own code taking maximum advantage of avaliable
//! features and degrading nicely, for your set of supported backends. See the backends' documentation for supported
//! feature sets.
//!
//! This entire module is `#![no_std]` compatible. See the crate root for more info.
//!
//! - By default:
//!   - 16 basic [`Color`]s (blue, green, cyan, red, magenta, yellow, black, and the bright equivalents)
//!   - Setting foreground and background

use alloc::string::String;

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
    /// All of the colors supported
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
        }
    }
}

/// The format of a single formatted item.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Format {
    /// The foreground color of the item
    pub fg: Color,
    /// The background color of the item
    pub bg: Color,
    /// Whether it's bolded or not
    pub bold: bool,
    /// Whether it's underlined or not
    pub underline: bool,
}

impl Format {
    /// The default formatting Redshell uses
    pub const NONE: Self = Format {
        fg: Color::White,
        bg: Color::Black,
        bold: false,
        underline: false,
    };
}

impl Default for Format {
    fn default() -> Self {
        Self::NONE
    }
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

/// Trait implemented by all formattable items (`Text` and `Cell`).
pub trait Formatted {
    fn get_fmt(&self) -> &Format;
    fn get_fmt_mut(&mut self) -> &mut Format;
}

/// Provides common formatting operations on anything implementing [`Formatted`].
pub trait FormattedExt: Formatted + Sized {
    /// Directly set the formatting of this item to some [`Format`]
    #[must_use]
    fn fmt(mut self, fmt: Format) -> Self {
        *self.get_fmt_mut() = fmt;
        self
    }

    /// Copy another item's formatting into this one.
    ///
    /// The two objects don't need to be the same type, e.g. you can copy a [`Text`]'s formatting to a [`Cell`].
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
        underline => underline = true,
        bold => bold = true,
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
            _fmt: $crate::fmt::Format,
        }
        impl $crate::fmt::Formatted for $name {
            fn get_fmt(&self) -> &$crate::fmt::Format {
                &self._fmt
            }
            fn get_fmt_mut(&mut self) -> &mut $crate::fmt::Format {
                &mut self._fmt
            }
        }
        impl $name {
            pub const fn of( $($field: $type),* ) -> Self {
                Self {
                    $( $field, )*
                    _fmt: $crate::fmt::Format::NONE,
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
    /// is designed to be used through `text!`, i.e. as a `Vec<Text>`.
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct Text {
        pub text: String,
    }
);

impl Text {
    pub fn plain(s: &str) -> Text {
        Text::of(s.into())
    }

    pub fn with_text(&self, new_text: String) -> Text {
        let mut res = self.clone();
        res.text = new_text.into();
        res
    }
}

/// Create a single [`Text`]. Not recommended to be used directly.
#[macro_export]
macro_rules! text1 {
    [
        $( $name:ident )*
        $text:literal
        $( ( $( $arg:expr ),* $(,)? ) )?
    ] => {
        {
            #[allow(unused_imports)]
            use $crate::fmt::{FormattedExt as _};
            $crate::fmt::Text::of(
                alloc::format!( $text $(, $( $arg ),* )? )
            ) $( . $name () )*
        }
    };
}

/// Create a series of formatted [`Text`]s.
#[macro_export]
macro_rules! text {
    [ $(
        $( $name:ident )*
        $text:literal
        $( ( $( $arg:expr ),* $(,)? ) )?
    ),* $(,)? ] => {
        {
            #[allow(unused_imports)]
            use $crate::fmt::{FormattedExt as _};
            alloc::vec![
                $(
                    $crate::fmt::Text::of(
                        alloc::format!( $text $(, $( $arg ),* )? )
                    ) $( . $name () )*
                ),*
            ]
        }
    };
}

fmt_type! {
    /// A single character that's been formatted. This is really only meant to be used in `Screen`.
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct Cell { pub ch: char }
}

/// Create a formatted [`Cell`].
#[macro_export]
macro_rules! cell {
    [ $( $name:ident )* $( $char:literal )? ] => {
        {
            #[allow(unused_imports)]
            use $crate::fmt::{FormattedExt as _};
            Cell::of($($char)?) $( .$name() )*
        }
    };
}

impl Cell {
    /// A blank cell with default formatting.
    pub const BLANK: Cell = cell!(' ');
}
