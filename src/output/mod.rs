//! This module handles all of the output, both abstractions and implementations.
//!
//! If you want to add more implementations, you need to:
//! - Add the relevant implementation of `Screen`, in a new submodule
//! - Modify `Screen::get` to properly detect and handle the new screen type, with `cfg!` or runtime checks

use std::{fmt, ops::{Add, AddAssign, Sub, SubAssign, Mul, MulAssign, Div, DivAssign}};

pub struct XY(usize, usize);

impl XY {
    pub fn x(&self) -> usize {
        self.0
    }

    pub fn y(&self) -> usize {
        self.1
    }
}

macro_rules! xy_op {
    ( $(
        $trait:ident($fn:ident) => $op:tt $assn_op:tt
    ),* $(,)? ) => {
        $(
            impl $trait for XY {
                type Output = XY;
                fn $fn(self, rhs: XY) -> XY {
                    XY(self.0 $op rhs.0, self.1 $op rhs.1)
                }
            }

            paste::paste! {
                impl [< $trait Assign >] for XY {
                    fn [< $fn _assign >] (&mut self, rhs: XY) {
                        self.0 $assn_op rhs.0;
                        self.1 $assn_op rhs.1;
                    }
                }
            }
        )*
    };
}

xy_op! {
    Add(add) => + +=,
    Sub(sub) => - -=,
    Mul(mul) => * *=,
    Div(div) => / /=,
}

impl fmt::Display for XY {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.0, self.1)
    }
}

/// The color of a piece of formatted text. Meant to be used through `Text` / `text!`. The numeric values are the ANSI
/// color codes for each color; that's also where the actual colors are from.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
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
}

/// A single bit of formatted text. Note this isn't really meant to be used on its own, though it can be; the API is
/// designed to be used through `text!`. To discourage direct use, the internals aren't documented.
#[derive(Clone)]
pub struct Text {
    pub text: String,
    pub fg: Color,
    pub bg: Color,
    pub bright: bool,
    pub bold: bool,
    pub underline: bool,
    pub italic: bool,
    pub invert: bool,
}

macro_rules! setters {
    ( $(
        $name:ident $( ( $($pname:ident: $ptype:ty),* $(,)? ) )?  => $field:ident = $value:expr
    ),* $(,)? ) => {
        $(
            pub fn $name(mut self $( , $( $pname: $ptype ),* )?) -> Self {
                self.$field = $value;
                self
            }
        )*
    };
}

impl Text {
    pub fn of(s: String) -> Text {
        Text {
            text: s,
            fg: Color::Default,
            bg: Color::Default,
            bright: false,
            bold: false,
            underline: false,
            italic: false,
            invert: false,
        }
    }

    setters! {
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
        bright => bright = true,        bold => bold = true,
        underline => underline = true,  italic => italic = true,
        invert => invert = true,
    }

    fn with_text(&self, new_text: String) -> Text {
        let mut new = self.clone();
        new.text = new_text;
        new
    }
}

impl fmt::Debug for Text {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Text {{ {:?}, ", self.text)?;
        if self.fg != Color::Default {
            write!(f, "fg: {:?}, ", self.fg)?;
        }
        if self.bg != Color::Default {
            write!(f, "bg: {:?}, ", self.bg)?;
        }
        if self.bright { write!(f, "bright: true, ")?; }
        if self.bold { write!(f, "bold: true, ")?; }
        if self.underline { write!(f, "underline: true, ")?; }
        if self.italic { write!(f, "italic: true, ")?; }
        if self.invert { write!(f, "invert: true, ")?; }
        write!(f, ".. }}")
    }
}

fn is_breakable(c: char) -> bool {
    c.is_whitespace() || c.is_control() || (c.is_ascii_punctuation() && c != '-' && c != '_')
}

fn wrap(chunks: Vec<Text>, width: usize, scroll: usize, indent: usize, first_indent: usize) -> Vec<Text> {
    let mut res = Vec::with_capacity(chunks.len());
    let mut line_num = 0;
    let mut col = first_indent;
    for item in chunks {
        for line in item.text.lines() {
            let mut rest = line;
            while rest.len() > width - col {
                let line = if let Some(break_pos) = rest[..width - col].find(is_breakable) {
                    &rest[..break_pos].trim_end()
                } else {
                    &rest[..width - col - 1]
                };

            }
        }
        if item.text.ends_with('\n') {
            line_num += 1;
            col = indent;
        }
    }
    res
}

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
                Text::of(
                    format!( $text $(, $( $arg ),* )? )
                ) $( . $name () )*
            ),+
        ]
    };
}

#[allow(unused)]
fn ensure_text_macro_compiles() {
    let name = "bloop";
    let _ = text!(
        red "hello",
        " there ",
        bold green on_blue "Liege {}!"(name),
        " You're a very {}"(name),
    );
}

/// A box of text which can be written to a `Screen`. Note these are meant to be generated on the fly, every frame,
/// possibly multiple times. They do the actual *writing* when they're dropped, converting the higher-level Textbox
/// API things to calls of `Screen::raw`.
pub struct Textbox<'a> {
    screen: &'a mut dyn Screen,
    chunks: Vec<Text>,
    pos: XY,
    size: XY,
    scroll: usize,
    indent: usize,
    first_indent: Option<usize>,
}

impl<'a> Textbox<'a> {
    setters! {
        pos(x: usize, y: usize) => pos = XY(x, y),
        size(x: usize, y: usize) => size = XY(x, y),
        scroll(amt: usize) => scroll = amt,
        indent(amt: usize) => indent = amt,
        first_indent(amt: usize) => first_indent = Some(amt),
    }
}

impl<'a> Drop for Textbox<'a> {
    fn drop(&mut self) {
        // TODO: Actually do the writing
    }
}

pub struct Header<'a> {
    screen: &'a mut dyn Screen,
    tabs: Vec<(String, usize)>,
    selected: Option<usize>,
    profile: String,
    time: String,
    lag: bool,
}

impl<'a> Header<'a> {
    pub fn tab(mut self, name: &str, notifs: usize) -> Self {
        self.tabs.push((name.into(), notifs));
        self
    }

    setters! {
        profile(name: &str) => profile = name.into(),
        time(now: &str) => time = now.into(),
        selected(tab: usize) => selected = Some(tab),
        lag(is: bool) => lag = is,
    }
}

impl<'a> Drop for Header<'a> {
    fn drop(&mut self) {
        let mut text = Vec::with_capacity(self.tabs.len() * 3 + 2);
        let mut width = 0;
        for (i, (name, notifs)) in self.tabs.iter().enumerate() {
            match self.selected {
                Some(n) if n == i => text.push(text1!(bold "{}"(name))),
                _ => text.push(text1!("{}"(name))),
            }
            if *notifs == 0 {
                text.push(text1!("   |"));
            } else if *notifs <= 9 {
                text.extend(text!(red " {} "(notifs), "|"));
            } else {
                text.extend(text!(red " + ", "|"));
            }
            width += name.len() + 5; // 5 for 3 spaces, one pipe, notifs char
        }
        text.push(text1!(" you are {}"(self.profile)));
        width += 9 + self.profile.len();
        let space_left = self.screen.size().x() - width;
        if self.lag {
            text.push(text1!(italic "{:>1$}"(self.time, space_left)));
        } else {
            text.push(text1!("{:>1$}"(self.time, space_left)));
        }
        self.screen.write_raw(text, XY(0, 0));
    }
}

/// The single common interface for all the various different screens -- to a console, to a GUI, etc. It also allows
/// for querying some metadata: size, etc. The Screen defines the *actual representation* of each color,
///
/// Note that nothing is written until `flush` is called; all of the other methods just edit the internal state. This
/// prevents any potential issues with flickering, partial updates being visible, etc.
///
/// Drop is required because *most* output will require some cleanup, e.g. resetting terminal state, and it's worth
/// forcing it to be explicitly ignored for the few cases where it matters so I don't forget when it does.
pub trait Screen: Drop {
    /// Get the size of the screen, as of this frame.
    /// (Don't worry too much about this changing midframe; hopefully resize detection will catch that)
    fn size(&self) -> XY;

    /// Blank out the entire screen -- no text visible, no formatting left over, etc.
    fn clear(&mut self);
    /// Directly write some text to the screen at the position. Does the bare minimum formatting, etc. May mishandle
    /// newlines, e.g. by directly writing them to the screen. It's expected that the higher-level methods will handle
    /// that appropriately.
    fn write_raw(&mut self, text: Vec<Text>, pos: XY);
    /// Actually display the changes.
    fn flush(&mut self);
}



impl dyn Screen {
    /// Get the default screen for the current configuration. May be compiled in, may be determined at runtime.
    /// Note this is meant to be run once, at startup; it also initializes the screen which may have one-time effects
    /// (e.g. setting standard input and output to raw mode).
    pub fn get() -> Box<dyn Screen> {
        Box::new(test::TestScreen)
    }

    /// Write a header to the screen. (Note this must be rewritten every frame!)
    pub fn header<'a>(&'a mut self) -> Header<'a> {
        Header {
            screen: self,
            tabs: Vec::with_capacity(5),
            selected: None,
            profile: "".into(),
            // TODO: Use an actual time type
            time: "".into(),
            lag: false,
        }
    }
    /// Write a text-box to the screen.
    pub fn textbox<'a>(&'a mut self, text: Vec<Text>) -> Textbox<'a> {
        Textbox {
            screen: self,
            chunks: text,
            pos: XY(0, 0),
            size: XY(0, 0),
            scroll: 0,
            indent: 0,
            first_indent: None,
        }
    }
}

mod test;
