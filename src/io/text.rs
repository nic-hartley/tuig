use rand::{seq::SliceRandom, Rng, prelude::Distribution, distributions::Standard};

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

impl Distribution<Color> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Color {
        *Color::all().choose(rng).unwrap()
    }
}

/// A single bit of formatted text. Note this isn't really meant to be used on its own, though it can be; the API is
/// designed to be used through `text!`. To discourage direct use, the internals aren't documented.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Text {
    pub text: String,
    pub fg: Color,
    pub bg: Color,
    pub bold: bool,
    pub underline: bool,
    pub invert: bool,
}

impl Text {
    pub fn of(s: String) -> Text {
        Text {
            text: s,
            fg: Color::Default,
            bg: Color::Default,
            bold: false,
            underline: false,
            invert: false,
        }
    }

    pub fn plain(s: &str) -> Text {
        Text::of(s.into())
    }

    crate::util::setters! {
        fg(c: Color) => fg = c,         bg(c: Color) => bg = c,
        black => fg = Color::Black,     on_black => bg = Color::Black,
        red => fg = Color::Red,         on_red => bg = Color::Red,
        green => fg = Color::Green,     on_green => bg = Color::Green,
        yellow => fg = Color::Yellow,   on_yellow => bg = Color::Yellow,
        blue => fg = Color::Blue,       on_blue => bg = Color::Blue,
        magenta => fg = Color::Magenta, on_magenta => bg = Color::Magenta,
        cyan => fg = Color::Cyan,       on_cyan => bg = Color::Cyan,
        white => fg = Color::White,     on_white => bg = Color::White,
        default => fg = Color::Default, on_default => bg = Color::Default,
        underline => underline = true,  bold => bold = true,
        invert => invert = true,
    }

    pub(super) fn with_text(&self, new_text: String) -> Text {
        let mut res = self.clone();
        res.text = new_text.into();
        res
    }
}

crate::util::abbrev_debug! {
    Text;
    write text,
    if fg != Color::Default,
    if bg != Color::Default,
    if bold != false,
    if underline != false,
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
        $( $name:ident )* $text:literal $( ( $( $arg:expr ),* $(,)? ) )?
    ),+ $(,)? ) => {
        vec![
            $(
                $crate::io::output::Text::of(
                    format!( $text $(, $( $arg ),* )? )
                ) $( . $name () )*
            ),+
        ]
    };
}
