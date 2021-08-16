//! This module handles all of the output, both abstractions and implementations.
//!
//! If you want to add more implementations, you need to:
//! - Add the relevant implementation of `Screen`, in a new submodule
//! - Modify `Screen::get` to properly detect and handle the new screen type, with `cfg!` or runtime checks

mod test;
mod ansi_cli;

use std::{fmt, ops::{Add, AddAssign, Sub, SubAssign, Mul, MulAssign, Div, DivAssign}};

#[derive(PartialEq, Eq, PartialOrd, Ord)]
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

impl fmt::Debug for XY {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "XY({}, {})", self.0, self.1)
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

macro_rules! abbrev_debug {
    (
        $class:ident $( < $( $lt:lifetime ),* > )?;
        $( write $always:ident, )*
        $( ignore $ignore:ident, )*
        $( if $sometimes:ident != $default:expr, )*
    ) => {
        impl $( < $( $lt ),* > )?  fmt::Debug for $class $( < $( $lt ),* > )? {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, concat!(stringify!($class), " {{ "))?;
                $(
                    write!(f, concat!(stringify!($always), ": {:?}, "), self.$always)?;
                )*
                $(
                    write!(f, concat!(stringify!($ignore), ": ..., "))?;
                )*
                $(
                    if self.$sometimes != $default {
                        write!(f, concat!(stringify!($sometimes), ": {:?}, "), self.$sometimes)?;
                    }
                )*
                write!(f, ".. }}")
            }
        }
    }
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
        let mut res = self.clone();
        res.text = new_text.into();
        res
    }
}

abbrev_debug! {
    Text;
    write text,
    if fg != Color::Default,
    if bg != Color::Default,
    if bright != false,
    if bold != false,
    if underline != false,
    if italic != false,
    if invert != false,
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

pub fn wrap_line(
    width: usize,
    mut line: &str,
    col: &mut usize, line_num: &mut usize,
    /*
    This parameter is basically just a dirty hack. If it's true, *col is effectively first_indent, so we should
    hyphenate words that are too long. If it's false, *col marks the end of the last bit of text, so we should
    wrap, even if it leaves a blank line.
    */
    mut starts_fresh: bool
) -> Vec<String> {
    assert!(*col < width);

    let mut lines = Vec::new();
    while line.len() > width - *col {
        let first_n = &line[..width - *col + 1];
        let (len, chunk) = if let Some(end_pos) = first_n.rfind(char::is_whitespace) {
            let len = match first_n[..end_pos].rfind(|c: char| !c.is_whitespace()) {
                Some(n) => n + 1, // rfind returns the address of the one, we want the one after
                None => first_n.len(), // not sure if this could happen but just in case
            };
            (len, first_n[..len].into())
        } else if starts_fresh {
            starts_fresh = false;
            (0, String::new())
        } else {
            (first_n.len()-2, format!("{}-", &first_n[..first_n.len()-2]))
        };
        lines.push(chunk);
        line = line[len..].trim_start();
        *col = 0;
        *line_num += 1;
    }
    *col = line.len();
    lines.push(line.into());
    lines
}

fn find_break(from: &str) -> Option<usize> {
    from.rfind(char::is_whitespace)
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

abbrev_debug! {
    Textbox<'a>;
    ignore chunks,
    if pos != XY(0, 0),
    if size != XY(0, 0),
    if scroll != 0,
    if indent != 0,
    if first_indent != None,
}

impl<'a> Drop for Textbox<'a> {
    fn drop(&mut self) {
        let first_indent = self.first_indent.unwrap_or(self.indent);
        let XY(width, height) = self.size;
        let XY(x, y) = self.pos;

        let mut col = first_indent;
        let mut line_num = 0;
        let mut line_start = true;

        macro_rules! next_line {
            ($new_para:expr) => {
                line_num += 1;
                line_start = true;
                col = if $new_para { first_indent } else { self.indent };
            }
        }

        macro_rules! write_raw {
            ($text:expr) => {
                if line_num >= self.scroll && line_num - self.scroll < height {
                    self.screen.write_raw(
                        $text,
                        XY(x + col, y + line_num - self.scroll)
                    );
                }
            }
        }

        macro_rules! do_wrap {
            ($chunk:ident, $line:ident) => {
                while $line.len() > width - col {
                    if let Some(break_pos) = find_break(&$line[..width - col]) {
                        let (subline, rest_of_line) = $line.split_at(break_pos);
                        $line = rest_of_line.trim_start();

                        write_raw!(vec![$chunk.with_text(subline.into())]);
                    } else if line_start {
                        // if we're already at the start of the line, can't exactly push stuff to the next line;
                        // that'd loop forever
                        let (subline, rest_of_line) = $line.split_at(width - col - 1);
                        $line = rest_of_line.trim_start();

                        write_raw!(vec![$chunk.with_text(subline.to_string() + "-")]);
                    } else {
                        // if we've just finished another chunk, so it's *not* the beginning of a line, then just go
                        // to the next line for anything that's too long
                        // (that means we don't do anything here; this branch is just for documentation)
                    }
                    next_line!(false);
                }
                if $line.len() > 0 {
                    write_raw!(vec![$chunk.with_text($line.into())]);
                    #[allow(unused_assignments)] {
                        col += $line.len();
                        line_start = false;
                    }
                }
            }
        }

        for chunk in &self.chunks {
            let mut rest = &chunk.text[..];
            while let Some(nl_pos) = rest.find('\n') {
                let (mut line, new_rest) = rest.split_at(nl_pos);
                rest = &new_rest[1..];

                do_wrap!(chunk, line);
                next_line!(true);
            }
            if !rest.is_empty() {
                do_wrap!(chunk, rest);
            } else {
                // ended with a newline, don't bother trying to format the zero remainin characters, that'll just
                // cause problems
                // (so just pass on to the next line)
            }
        }
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
pub trait Screen {
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
        Box::new(test::TestScreen::get())
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
