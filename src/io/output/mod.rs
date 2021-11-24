//! This module handles all of the output, both abstractions and implementations.
//!
//! If you want to add more implementations, you need to:
//! - Add the relevant implementation of `Screen`, in a new submodule
//! - Modify `Screen::get` to properly detect and handle the new screen type, with `cfg!` or runtime checks

mod test;
mod ansi_cli;

use std::fmt;

use crate::io::XY;

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
    BrightBlack = 60,
    BrightRed = 61,
    BrightGreen = 62,
    BrightYellow = 63,
    BrightBlue = 64,
    BrightMagenta = 65,
    BrightCyan = 66,
    BrightWhite = 67,
}

/// A single bit of formatted text. Note this isn't really meant to be used on its own, though it can be; the API is
/// designed to be used through `text!`. To discourage direct use, the internals aren't documented.
#[derive(Clone)]
pub struct Text {
    pub text: String,
    pub fg: Color,
    pub bg: Color,
    pub bold: bool,
    pub underline: bool,
}

impl Text {
    pub fn of(s: String) -> Text {
        Text {
            text: s,
            fg: Color::Default,
            bg: Color::Default,
            bold: false,
            underline: false,
        }
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
    }

    fn with_text(&self, new_text: String) -> Text {
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
    crate::util::setters! {
        pos(x: usize, y: usize) => pos = XY(x, y),
        size(x: usize, y: usize) => size = XY(x, y),
        scroll(amt: usize) => scroll = amt,
        indent(amt: usize) => indent = amt,
        first_indent(amt: usize) => first_indent = Some(amt),
    }
}

crate::util::abbrev_debug! {
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

        assert!(width > self.indent);
        assert!(width > first_indent);

        let mut col = first_indent;
        let mut line_num = 0;
        let mut line_start = true;

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

        macro_rules! next_line {
            ($new_para:expr) => {
                line_num += 1;
                line_start = true;
                col = if $new_para { first_indent } else { self.indent };
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
                // ended with a newline, don't bother trying to format the zero remaining characters, that'll just
                // cause problems (so just pass on to the next line)
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
}

impl<'a> Header<'a> {
    pub fn tab(mut self, name: &str, notifs: usize) -> Self {
        self.tabs.push((name.into(), notifs));
        self
    }

    crate::util::setters! {
        profile(name: &str) => profile = name.into(),
        time(now: &str) => time = now.into(),
        selected(tab: usize) => selected = Some(tab),
    }
}

crate::util::abbrev_debug! {
    Header<'a>;
    if tabs != vec![],
    write selected,
    write profile,
    write time,
}

impl<'a> Drop for Header<'a> {
    fn drop(&mut self) {
        let mut text = Vec::with_capacity(self.tabs.len() * 3 + 1);
        let mut width = 0;
        for (i, (name, notifs)) in self.tabs.iter().enumerate() {
            match self.selected {
                Some(n) if n == i => text.push(Text::of(name.into()).underline()),
                _ => text.push(Text::of(name.into())),
            }
            if *notifs == 0 {
                text.push(text1!("   | "));
            } else if *notifs <= 9 {
                text.extend(text!(red " {} "(notifs), "| "));
            } else {
                text.extend(text!(red " + ", "| "));
            }
            width += name.len() + 5; // 5 for " n | "
        }

        text.push(text1!("you are {}"(self.profile)));
        width += 8 + self.profile.len();

        // this weird construction ensures that, if we manually highlight the header, the whole line gets highlighted
        // and doesn't have any weird gaps.
        let space_left = self.screen.size().x() - width;
        text.push(text1!("{:>1$}"(self.time, space_left)));
        self.screen.write_raw(text, XY(0, 0));
    }
}

/// The single common interface for all the various different screens -- to a console, to a GUI, etc. It also allows
/// for querying some metadata: size, etc. The Screen defines the *actual representation* of each color,
///
/// Note that nothing is written until `flush` is called; all of the other methods just edit the internal state. This
/// prevents any potential issues with flickering, partial updates being visible, etc.
///
/// `flush` is called once every frame, *after* everything has rendered. It 
pub trait Screen {
    /// Get the size of the screen, as of this frame.
    /// (Don't worry too much about this changing midframe; hopefully resize detection will catch that and hide issues)
    fn size(&self) -> XY;

    /// Directly write some text to the screen at the position. Does the bare minimum formatting, etc. May mishandle
    /// special chars, e.g. by directly writing them to the console. It's expected that the higher-level methods will
    /// handle that appropriately.
    fn write_raw(&mut self, text: Vec<Text>, pos: XY);
    /// Clear the screen and draw the next frame's worth of stuff.
    fn flush(&mut self);
    /// Just clear the (cached write_raw) screen; used to keep the screen relatively smooth even when it's resized.
    /// Note this **should not** actually send the clear command to the screen.
    fn clear(&mut self);
    /// Actually clear the screen. Used, e.g., when resizing is detected, to prevent weird shearing issues.
    fn do_clear(&mut self);
}

impl dyn Screen + '_ {
    /// Get the default screen for the current configuration. May be compiled in, may be determined at runtime.
    /// Note this is meant to be run once, at startup; it also initializes the screen which may have one-time effects
    /// (e.g. setting standard input and output to raw mode).
    pub fn get() -> Box<dyn Screen> {
        if cfg!(feature = "force_out_test") {
            return Box::new(test::TestScreen::get());
        }
        if cfg!(feature = "force_ansi_test") {
            return Box::new(ansi_cli::AnsiScreen::get().expect("Failed to initialize forced ANSI CLI output."));
        }

        if let Ok(s) = ansi_cli::AnsiScreen::get() {
            return Box::new(s);
        }
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
