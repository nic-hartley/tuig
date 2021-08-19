use std::{io::{Write, stdout}, mem};

use crossterm::{execute, terminal::{self, DisableLineWrap, EnableLineWrap, EnterAlternateScreen, LeaveAlternateScreen}};

use super::*;

// TODO: Replace all the `unwrap`s with 

pub struct AnsiScreen {
    /// The text fragments that will be written to screen, plus its horizontal position; one vec per line.
    /// Each line is sorted from left to right (lower to higher X).
    texts: Vec<Vec<(usize, Text)>>,
}

impl AnsiScreen {
    pub fn get() -> crossterm::Result<Self> {
        terminal::enable_raw_mode()?;
        execute!(stdout(), EnterAlternateScreen, DisableLineWrap)?;
        Ok(Self { texts: vec![] })
    }
}

impl Drop for AnsiScreen {
    fn drop(&mut self) {
        execute!(stdout(), EnableLineWrap, LeaveAlternateScreen).unwrap();
        terminal::disable_raw_mode().unwrap();
    }
}

impl Screen for AnsiScreen {
    fn size(&self) -> XY {
        let (x, y) = terminal::size().unwrap();
        XY(x as usize, y as usize)
    }

    fn write_raw(&mut self, text: Vec<Text>, pos: XY) {
        let XY(width, height) = self.size();
        if pos.y() > height || pos.x() > width {
            println!("entirely oob");
            return;
        }
        if self.texts.len() != height {
            println!("had to resize terminal");
            self.texts.resize_with(height, || vec![]);
        }

        // TODO: Trim overlaps and actually ensure things are sorted
        // (for now it just takes advantage of timing to avoid issues)
        let mut col = pos.x();
        for t in text {
            if col >= width {
                break;
            }
            // don't need to trim: no line wrapping in in this screen
            let len = t.text.len();
            self.texts[pos.y()].push((col, t));
            col += len;
        }
    }

    fn flush(&mut self) {
        // replace so we can take ownership and .into_iter() instead of .drain()ing
        let lines = mem::replace(&mut self.texts, vec![]);
        let mut out = Vec::<u8>::new();
        write!(out, "\x1b[2J\x1b[0;0H").unwrap();
        for line in lines {
            // TODO: update to use crossterm properly
            write!(out, "\n").unwrap();
            for (pos, text) in line {
                write!(out, "\x1b[{}G\x1b[{};{};{};{}m{}",
                    pos + 1,
                    text.fg as usize + 30,
                    text.bg as usize + 40,
                    if text.bold { 1 } else { 22 },
                    if text.underline { 4 } else { 24 },
                    text.text,
                ).unwrap();
            }
        }
        stdout().write_all(&mut out).unwrap();
    }

    fn clear(&mut self) {
        self.texts.clear()
    }

    fn do_clear(&mut self) {
        stdout().write_all("\x1b[2J".as_bytes()).unwrap();
    }
}
