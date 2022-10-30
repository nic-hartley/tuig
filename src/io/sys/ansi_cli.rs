use std::{io, mem, time::Duration};

use crossterm::{
    cursor::{Hide, MoveDown, MoveTo, MoveToColumn, Show},
    event::{self as ct, DisableMouseCapture, EnableMouseCapture},
    execute,
    style::{
        Attribute, Color as CrosstermColor, ResetColor, SetAttribute, SetAttributes,
        SetBackgroundColor, SetForegroundColor,
    },
    terminal::{
        self, Clear, ClearType, DisableLineWrap, EnableLineWrap, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use tokio::{
    io::AsyncWriteExt,
    sync::{mpsc, oneshot},
};

use crate::io::{
    input::{Action, Key, MouseButton},
    output::Screen,
    output::{Cell, Color as RedshellColor},
    XY,
};

use super::IoSystem;

macro_rules! mods {
    ( $mods:ident, $action:ident ) => {
        if $mods.contains(ct::KeyModifiers::SHIFT) {
            try_send!($action {
                key: Key::LeftShift
            });
        }
        if $mods.contains(ct::KeyModifiers::CONTROL) {
            try_send!($action { key: Key::LeftCtrl });
        }
        if $mods.contains(ct::KeyModifiers::ALT) {
            try_send!($action { key: Key::LeftAlt });
        }
    };
}

fn io4ct_btn(ct: ct::MouseButton) -> MouseButton {
    match ct {
        ct::MouseButton::Left => MouseButton::Left,
        ct::MouseButton::Middle => MouseButton::Middle,
        ct::MouseButton::Right => MouseButton::Right,
    }
}

fn process_input(actions: mpsc::UnboundedSender<Action>, mut stop: oneshot::Receiver<()>) {
    macro_rules! try_send {
        ( $type:ident $( ($nt:expr) )? $( { $($br:tt)* } )? ) => {
            match actions.send(Action::$type $(($nt))? $({$($br)*})? ) {
                Ok(_) => (),
                Err(_) => return,
            }
        }
    }
    loop {
        match stop.try_recv() {
            Err(oneshot::error::TryRecvError::Empty) => (),
            _ => return,
        }
        match crossterm::event::poll(Duration::from_millis(100)) {
            Ok(false) => continue,
            Ok(true) => (),
            Err(e) => {
                try_send!(Error(format!("polling: {}", e)));
                return;
            }
        }
        let ev = match crossterm::event::read() {
            Ok(ev) => ev,
            Err(e) => {
                try_send!(Error(format!("polling: {}", e)));
                return;
            }
        };
        match ev {
            ct::Event::Key(ct::KeyEvent { code, modifiers }) => {
                mods!(modifiers, KeyPress);
                if code == ct::KeyCode::BackTab {
                    try_send!(KeyPress {
                        key: Key::LeftShift
                    });
                    try_send!(KeyPress { key: Key::Tab });
                    try_send!(KeyRelease { key: Key::Tab });
                    try_send!(KeyRelease {
                        key: Key::LeftShift
                    });
                } else if code == ct::KeyCode::Null {
                    try_send!(Unknown("null character".into()));
                } else {
                    let action_code = match code {
                        ct::KeyCode::Char(c) => Key::Char(c),
                        ct::KeyCode::F(c) => Key::F(c),
                        ct::KeyCode::Backspace => Key::Backspace,
                        ct::KeyCode::Enter => Key::Enter,
                        ct::KeyCode::Left => Key::Left,
                        ct::KeyCode::Right => Key::Right,
                        ct::KeyCode::Up => Key::Up,
                        ct::KeyCode::Down => Key::Down,
                        ct::KeyCode::Home => Key::Home,
                        ct::KeyCode::End => Key::End,
                        ct::KeyCode::PageUp => Key::PageUp,
                        ct::KeyCode::PageDown => Key::PageDown,
                        ct::KeyCode::Tab => Key::Tab,
                        ct::KeyCode::Delete => Key::Delete,
                        ct::KeyCode::Insert => Key::Insert,
                        ct::KeyCode::Esc => Key::Escape,
                        kc => unreachable!("unhandled keycode {:?}; should be handled earlier", kc),
                    };
                    try_send!(KeyPress { key: action_code });
                    try_send!(KeyRelease { key: action_code });
                }
                mods!(modifiers, KeyRelease);
            }
            ct::Event::Resize(..) => (), // handled by polling in the output when it's requested
            ct::Event::Mouse(ct::MouseEvent {
                row,
                column: col,
                kind,
                modifiers,
            }) => {
                mods!(modifiers, KeyPress);
                let pos = XY(col as usize, row as usize);
                match kind {
                    ct::MouseEventKind::Up(btn) => try_send!(MouseRelease {
                        button: io4ct_btn(btn),
                        pos
                    }),
                    ct::MouseEventKind::Down(btn) => try_send!(MousePress {
                        button: io4ct_btn(btn),
                        pos
                    }),
                    ct::MouseEventKind::Drag(btn) => try_send!(MouseMove {
                        button: Some(io4ct_btn(btn)),
                        pos
                    }),
                    ct::MouseEventKind::Moved => try_send!(MouseMove { button: None, pos }),
                    ct::MouseEventKind::ScrollUp => {
                        try_send!(MousePress {
                            button: MouseButton::ScrollUp,
                            pos
                        });
                        try_send!(MousePress {
                            button: MouseButton::ScrollUp,
                            pos
                        });
                    }
                    ct::MouseEventKind::ScrollDown => {
                        try_send!(MousePress {
                            button: MouseButton::ScrollDown,
                            pos
                        });
                        try_send!(MousePress {
                            button: MouseButton::ScrollDown,
                            pos
                        });
                    }
                }
                mods!(modifiers, KeyRelease);
            }
        };
    }
}

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
    let mut attrs = [
        Attribute::NormalIntensity,
        Attribute::NoUnderline,
        Attribute::NoReverse,
    ];
    if bold {
        attrs[0] = Attribute::Bold;
    }
    if underline {
        attrs[1] = Attribute::Underlined;
    }
    if invert {
        attrs[2] = Attribute::Reverse;
    }
    crossterm::queue!(
        &mut out,
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
            let attr = if bold {
                Attribute::Bold
            } else {
                Attribute::NormalIntensity
            };
            crossterm::queue!(&mut out, SetAttribute(attr))?;
        }
        if cell.underline != underline {
            underline = cell.underline;
            let attr = if underline {
                Attribute::Underlined
            } else {
                Attribute::NoUnderline
            };
            crossterm::queue!(&mut out, SetAttribute(attr))?;
        }
        if cell.invert != invert {
            invert = cell.invert;
            let attr = if invert {
                Attribute::Reverse
            } else {
                Attribute::NoReverse
            };
            crossterm::queue!(&mut out, SetAttribute(attr))?;
        }
        out.extend_from_slice(cell.ch.encode_utf8(&mut ch_b).as_bytes());
    }
    crossterm::queue!(&mut out, MoveDown(1), MoveToColumn(0),)?;

    Ok(out)
}

pub struct AnsiScreen {
    queue: mpsc::UnboundedReceiver<Action>,
    stop: Option<oneshot::Sender<()>>,
}

impl AnsiScreen {
    fn init_term() -> crossterm::Result<()> {
        terminal::enable_raw_mode()?;
        execute!(
            std::io::stdout(),
            EnableMouseCapture,
            EnterAlternateScreen,
            DisableLineWrap,
            Hide,
            Clear(ClearType::All),
        )?;
        Ok(())
    }

    fn clean_term() -> crossterm::Result<()> {
        execute!(
            std::io::stdout(),
            Clear(ClearType::All),
            Show,
            EnableLineWrap,
            LeaveAlternateScreen,
            DisableMouseCapture,
        )?;
        terminal::disable_raw_mode()?;
        Ok(())
    }

    pub fn get() -> crossterm::Result<Self> {
        Self::init_term()?;
        std::panic::set_hook(Box::new(|i| {
            let _ = Self::clean_term();
            println!("{}", i);
            let _ = Self::init_term();
        }));
        let (queue_s, queue_r) = mpsc::unbounded_channel();
        let (stop_s, stop_r) = oneshot::channel();
        tokio::task::spawn_blocking(|| process_input(queue_s, stop_r));
        Ok(Self {
            queue: queue_r,
            stop: Some(stop_s),
        })
    }
}

impl Drop for AnsiScreen {
    fn drop(&mut self) {
        let stop = mem::replace(&mut self.stop, None).expect("already dropped");
        // if the receiver is dead, that's fine; that means the queue won't have anything
        // else put in it. (and it can happen if e.g. there's an IO error)
        let _ = stop.send(());
        while self.queue.try_recv() != Err(mpsc::error::TryRecvError::Disconnected) {
            // flushing the queue and waiting for it to disconnect in the condition itself
        }
        Self::clean_term().expect("failed to clean up terminal");
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

    async fn input(&mut self) -> io::Result<crate::io::input::Action> {
        Ok(self.queue.recv().await.expect("unexpected queue closure"))
    }
}
