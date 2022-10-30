use std::{io::{Write, stdout}, mem};

use crossterm::{cursor::{Hide, Show}, execute, terminal::{self, Clear, ClearType, DisableLineWrap, EnableLineWrap, EnterAlternateScreen, LeaveAlternateScreen}};

use crate::io::XY;

use super::{Text, Screen};

// TODO: Replace all the `unwrap`s with 

pub struct AnsiScreen {
    /// The text fragments that will be written to screen, plus its horizontal position; one vec per line.
    /// Each line is sorted from left to right (lower to higher X).
    texts: Vec<Vec<(usize, Text)>>,
}

impl AnsiScreen {
    pub fn get() -> crossterm::Result<Self> {
        execute!(stdout(),
            EnterAlternateScreen,
            DisableLineWrap,
            Hide,
            Clear(ClearType::All)
        )?;
        std::panic::set_hook(Box::new(|i| {
            let _ = execute!(stdout(),
                Clear(ClearType::All),
                Show,
                EnableLineWrap,
                LeaveAlternateScreen
            );
            println!("{}", i);
            let _ = execute!(stdout(),
                EnterAlternateScreen,
                DisableLineWrap,
                Hide,
                Clear(ClearType::All)
            );
        }));
        Ok(Self { texts: vec![] })
    }
}

impl Drop for AnsiScreen {
    fn drop(&mut self) {
        execute!(stdout(),
            Clear(ClearType::All),
            Show,
            EnableLineWrap,
            LeaveAlternateScreen
        ).unwrap();
        terminal::disable_raw_mode().unwrap();
    }
}

#[async_trait::async_trait]
impl Screen for AnsiScreen {
    fn size(&self) -> XY {
        let (x, y) = terminal::size().unwrap();
        XY(x as usize, y as usize)
    }

    fn write_raw(&mut self, text: Vec<Text>, pos: XY) {
        let XY(width, height) = self.size();
        if pos.y() > height || pos.x() > width {
            return;
        }
        if self.texts.len() != height {
            self.texts.resize_with(height, || vec![]);
        }

        // TODO: Trim overlaps and actually ensure things are sorted
        // (for now it just takes advantage of 'timing' to avoid issues)
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

    async fn flush(&mut self) {
        // replace so we can take ownership and .into_iter() instead of .drain()ing
        let lines = mem::replace(&mut self.texts, vec![]);
        let mut out = Vec::<u8>::new();
        write!(out, "\x1b[2J\x1b[0;0H").unwrap();
        for line in lines {
            // TODO: update to use crossterm properly
            write!(out, "\n").unwrap();
            for (pos, text) in line {
                // TODO: minimize the changes and data sent
                // (e.g. calculate distance, pick spaces or escape codes)
                // (e.g. detect what's staying the same and don't re-set it)
                write!(out, "\x1b[{}G\x1b[0;{};{};{};{}{}m{}",
                    pos + 1,
                    text.fg as usize + 30,
                    text.bg as usize + 40,
                    if text.bold { 1 } else { 22 },
                    if text.underline { 4 } else { 24 },
                    if text.invert { ";7" } else { "" },
                    text.text,
                ).unwrap();
            }
        }
        stdout().write_all(&mut out).unwrap();
    }

    fn clear(&mut self) {
        self.texts.clear()
    }

    async fn clear_raw(&mut self) {
        stdout().write_all("\x1b[2J".as_bytes()).unwrap();
    }
}
