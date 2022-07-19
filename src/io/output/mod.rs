//! This module handles all of the output, both abstractions and implementations.
//!
//! If you want to add more implementations, you need to:
//! - Add the relevant implementation of `Screen`, in a new submodule
//! - Modify `Screen::get` to properly detect and handle the new screen type, with `cfg!` or runtime checks

pub mod test;
pub mod ansi_cli;

use crate::io::XY;

mod text;
pub use text::*;
mod widgets;
pub use widgets::*;

/// The single common interface for all the various different screens -- to a console, to a GUI, etc. It also allows
/// for querying some metadata: size, etc. The Screen defines the *actual representation* of each color,
///
/// Note that nothing is written until `flush` is called; all of the other methods just edit the internal state. This
/// prevents any potential issues with flickering, partial updates being visible, etc.
///
/// `flush` is called once every frame, *after* everything has rendered. It's the only thing that should be doing any
/// actual IO with the screen; everything else should just be caching the frame info.
#[async_trait::async_trait]
pub trait Screen {
    /// Get the size of the screen, as of this frame.
    /// (Don't worry too much about this changing midframe; hopefully resize detection will catch that and hide issues)
    fn size(&self) -> XY;

    /// Directly write a single text element to the screen. By default, just calls [`Screen::write_raw`] with a single
    /// element [`Vec`], but should generally be overridden to avoid the Vec overhead when reasonable.
    fn write_raw_single(&mut self, text: Text, pos: XY) {
        self.write_raw(vec![text], pos)
    }
    /// Directly write some text to the screen at the position. Does the bare minimum formatting, etc. May mishandle
    /// special chars, e.g. by directly writing them to the console. It's expected that the higher-level methods will
    /// handle that appropriately.
    fn write_raw(&mut self, text: Vec<Text>, pos: XY);
    /// Clear the actual screen and draw the next frame's worth of stuff on it.
    async fn flush(&mut self);
    /// Just clear the (cached write_raw) screen; used to keep the screen relatively smooth even when it's resized.
    /// Note this **should not** actually send the clear command to the screen.
    fn clear(&mut self);
    /// Actually clear the screen. Used, e.g., when resizing is detected, to prevent weird shearing issues.
    async fn clear_raw(&mut self);
}

impl dyn Screen + '_ {
    /// Get the default screen for the current configuration. May be compiled in, may be determined at runtime.
    /// Note this is meant to be run once, at startup; it also initializes the screen which may have one-time effects
    /// (e.g. setting standard input and output to raw mode).
    pub fn get() -> Box<dyn Screen> {
        if cfg!(feature = "force_out_test") {
            return Box::new(test::TestScreen::get());
        }
        if cfg!(feature = "force_out_ansi") {
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
            width: None,
            height: None,
            scroll: 0,
            indent: 0,
            first_indent: None,
        }
    }

    pub fn vertical<'a>(&'a mut self, col: usize) -> Vertical<'a> {
        Vertical {
            screen: self,
            col,
            start: None,
            end: None,
            char: '|',
        }
    }

    pub fn horizontal<'a>(&'a mut self, row: usize) -> Horizontal<'a> {
        Horizontal {
            screen: self,
            row,
            start: None,
            end: None,
            char: '-',
        }
    }
}
