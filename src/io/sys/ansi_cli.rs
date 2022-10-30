use std::io;

use crossterm::{cursor::{Hide, Show, MoveTo, MoveToColumn, MoveDown}, execute, terminal::{self, Clear, ClearType, DisableLineWrap, EnableLineWrap, EnterAlternateScreen, LeaveAlternateScreen}, style::{SetForegroundColor, Color as CrosstermColor, SetBackgroundColor, SetAttribute, Attribute, SetAttributes, ResetColor}};
use tokio::io::AsyncWriteExt;

use crate::io::{XY, output::Screen, output::{Color as RedshellColor, Cell}};

use super::IoSystem;

fn ct4rs_color(rs: RedshellColor) -> CrosstermColor {
    match rs {
        RedshellColor::BrightBlack => CrosstermColor::DarkGrey,
        RedshellColor::Black => CrosstermColor::Black,
        RedshellColor::BrightRed => CrosstermColor::Red,
        RedshellColor::Red => CrosstermColor::DarkRed,
        RedshellColor::BrightGreen => CrosstermColor::Green,
        RedshellColor::Green => CrosstermColor::DarkGreen,
        RedshellColor::BrightYellow => CrosstermColor::Yellow,
        RedshellColor::Yellow => CrosstermColor::DarkYellow,
        RedshellColor::BrightBlue => CrosstermColor::Blue,
        RedshellColor::Blue => CrosstermColor::DarkBlue,
        RedshellColor::BrightMagenta => CrosstermColor::Magenta,
        RedshellColor::Magenta => CrosstermColor::DarkMagenta,
        RedshellColor::BrightCyan => CrosstermColor::Cyan,
        RedshellColor::Cyan => CrosstermColor::DarkCyan,
        RedshellColor::BrightWhite => CrosstermColor::White,
        RedshellColor::White => CrosstermColor::Grey,
        RedshellColor::Default => CrosstermColor::Reset,
    }
}

fn render_row(row: &[Cell]) -> io::Result<Vec<u8>> {
    let mut out = vec![];

    let mut ch_b = [0u8; 4];

    let mut fg = row[0].fg;
    let mut bg = row[0].bg;
    let mut bold = row[0].bold;
    let mut underline = row[0].underline;
    let mut invert = row[0].invert;
    let mut attrs = [Attribute::NormalIntensity, Attribute::NoUnderline, Attribute::NoReverse];
    if bold {
        attrs[0] = Attribute::Bold;
    }
    if underline {
        attrs[1] = Attribute::Underlined;
    }
    if invert {
        attrs[2] = Attribute::Reverse;
    }
    crossterm::queue!(&mut out,
        ResetColor,
        SetForegroundColor(ct4rs_color(fg)),
        SetBackgroundColor(ct4rs_color(bg)),
        SetAttribute(Attribute::Reset),
        SetAttributes(attrs.as_ref().into()),
    )?;
    out.extend_from_slice(row[0].ch.encode_utf8(&mut ch_b).as_bytes());

    for cell in &row[1..] {
        if cell.fg != fg {
            fg = cell.fg;
            crossterm::queue!(&mut out, SetForegroundColor(ct4rs_color(fg)))?;
        }
        if cell.bg != bg {
            bg = cell.bg;
            crossterm::queue!(&mut out, SetBackgroundColor(ct4rs_color(bg)))?;
        }
        if cell.bold != bold {
            bold = cell.bold;
            let attr = if bold { Attribute::Bold } else { Attribute::NormalIntensity };
            crossterm::queue!(&mut out, SetAttribute(attr))?;
        }
        if cell.underline != underline {
            underline = cell.underline;
            let attr = if underline { Attribute::Underlined } else { Attribute::NoUnderline };
            crossterm::queue!(&mut out, SetAttribute(attr))?;
        }
        if cell.invert != invert {
            invert = cell.invert;
            let attr = if invert { Attribute::Reverse } else { Attribute::NoReverse };
            crossterm::queue!(&mut out, SetAttribute(attr))?;
        }
        out.extend_from_slice(cell.ch.encode_utf8(&mut ch_b).as_bytes());
    }
    crossterm::queue!(&mut out,
        MoveDown(1),
        MoveToColumn(0),
    )?;

    Ok(out)
}

// No need to store anything -- this only ever reads/writes to stdin/stdout.
pub struct AnsiScreen;

impl AnsiScreen {
    pub fn get() -> crossterm::Result<Self> {
        execute!(std::io::stdout(),
            EnterAlternateScreen,
            DisableLineWrap,
            Hide,
            Clear(ClearType::All)
        )?;
        std::panic::set_hook(Box::new(|i| {
            let _ = execute!(std::io::stdout(),
                Clear(ClearType::All),
                Show,
                EnableLineWrap,
                LeaveAlternateScreen
            );
            println!("{}", i);
            let _ = execute!(std::io::stdout(),
                EnterAlternateScreen,
                DisableLineWrap,
                Hide,
                Clear(ClearType::All)
            );
        }));
        Ok(Self)
    }
}

impl Drop for AnsiScreen {
    fn drop(&mut self) {
        execute!(std::io::stdout(),
            Clear(ClearType::All),
            Show,
            EnableLineWrap,
            LeaveAlternateScreen
        ).unwrap();
        terminal::disable_raw_mode().unwrap();
    }
}

#[async_trait::async_trait]
impl IoSystem for AnsiScreen {
    fn size(&self) -> XY {
        let (x, y) = terminal::size().unwrap();
        XY(x as usize, y as usize)
    }

    async fn draw(&mut self, screen: &Screen) -> io::Result<()> {
        let out = tokio::task::block_in_place(|| -> io::Result<Vec<u8>> {
            let mut out = vec![];
            crossterm::queue!(&mut out, MoveTo(0, 0))?;
            for row in screen.rows() {
                out.extend(render_row(row)?);
            }
            Ok(out)
        })?;
        let mut stdout = tokio::io::stdout();
        stdout.write_all(&out).await?;
        stdout.flush().await
    }

    async fn input(&mut self) -> std::io::Result<crate::io::input::Action> {
        todo!()
    }
}
