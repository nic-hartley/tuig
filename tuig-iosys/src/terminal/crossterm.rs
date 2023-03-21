//! Implements the (crossterm-based) rendering to CLI.

#![cfg(feature = "cli_crossterm")]

#[cfg(not(feature = "std"))]
compile_error!("enable std to use cli_crossterm");

use std::{
    io::Write,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, RecvError, TryRecvError},
        Arc,
    },
    time::Duration,
};

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

use crate::{
    action::{Action, Key, MouseButton},
    fmt::{Cell, Color as RsColor, Formatted},
    screen::Screen,
    xy::XY,
    IoRunner, IoSystem,
};

fn io4ct_btn(ct: ct::MouseButton) -> MouseButton {
    match ct {
        ct::MouseButton::Left => MouseButton::Left,
        ct::MouseButton::Middle => MouseButton::Middle,
        ct::MouseButton::Right => MouseButton::Right,
    }
}

pub struct CtRunner {
    actions: mpsc::Sender<Action>,
    stop: Arc<AtomicBool>,
}

impl CtRunner {
    fn init_term() -> crate::Result<()> {
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

    fn clean_term() -> crate::Result<()> {
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

    fn new(actions: mpsc::Sender<Action>, stop: Arc<AtomicBool>) -> crate::Result<Self> {
        Self::init_term()?;
        std::panic::set_hook(Box::new(|i| {
            let _ = Self::clean_term();
            println!("{}", i);
            // set back up in preparation for drop
            #[cfg(panic = "unwind")]
            let _ = Self::init_term();
        }));
        Ok(Self { actions, stop })
    }
}

impl Drop for CtRunner {
    fn drop(&mut self) {
        let _ = Self::clean_term();
    }
}

impl IoRunner for CtRunner {
    fn step(&mut self) -> bool {
        // check whether we've been told to stop
        if self.stop.load(Ordering::Relaxed) {
            writeln!(std::io::stderr(), "closing gracefully-ish").unwrap();
            return true;
        }

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
        macro_rules! try_send {
            ( $type:ident $( ($nt:expr) )? $( { $($br:tt)* } )? ) => {
                match self.actions.send(Action::$type $(($nt))? $({$($br)*})? ) {
                    Ok(_) => (),
                    Err(_) => return true,
                }
            }
        }
        // get an event from the terminal
        // (zero timeout to avoid blocking in `step`)
        match crossterm::event::poll(Duration::ZERO) {
            Ok(false) => return false,
            Ok(true) => (),
            Err(e) => {
                try_send!(Error(format!("polling: {}", e)));
                return true;
            }
        }
        // we have an event, so get it
        let ev = match crossterm::event::read() {
            Ok(ev) => ev,
            Err(e) => {
                try_send!(Error(format!("polling: {}", e)));
                return true;
            }
        };
        // process the event into a redshell `Event`
        match ev {
            ct::Event::Key(ct::KeyEvent {
                code, modifiers, ..
            }) => {
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
                        ct::KeyCode::F(c) => Key::F(c as usize),
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
            ct::Event::Resize(..) => try_send!(Redraw),
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
                        button: io4ct_btn(btn)
                    }),
                    ct::MouseEventKind::Down(btn) => try_send!(MousePress {
                        button: io4ct_btn(btn)
                    }),
                    ct::MouseEventKind::Drag(_) => try_send!(MouseMove { pos }),
                    ct::MouseEventKind::Moved => try_send!(MouseMove { pos }),
                    ct::MouseEventKind::ScrollUp => {
                        try_send!(MousePress {
                            button: MouseButton::ScrollUp
                        });
                        try_send!(MousePress {
                            button: MouseButton::ScrollUp
                        });
                    }
                    ct::MouseEventKind::ScrollDown => {
                        try_send!(MousePress {
                            button: MouseButton::ScrollDown
                        });
                        try_send!(MousePress {
                            button: MouseButton::ScrollDown
                        });
                    }
                }
                mods!(modifiers, KeyRelease);
            }
            ct::Event::FocusGained => try_send!(Redraw),
            ct::Event::FocusLost => (),
            ct::Event::Paste(_) => unreachable!("bracketed paste not enabled yet"), // TODO: #40
        };

        false
    }
}

/// Crossterm color for Redshell colors
fn ct4rs_color(rs: RsColor) -> CrosstermColor {
    match rs {
        RsColor::BrightBlack => CrosstermColor::DarkGrey,
        RsColor::Black => CrosstermColor::Black,
        RsColor::BrightRed => CrosstermColor::Red,
        RsColor::Red => CrosstermColor::DarkRed,
        RsColor::BrightGreen => CrosstermColor::Green,
        RsColor::Green => CrosstermColor::DarkGreen,
        RsColor::BrightYellow => CrosstermColor::Yellow,
        RsColor::Yellow => CrosstermColor::DarkYellow,
        RsColor::BrightBlue => CrosstermColor::Blue,
        RsColor::Blue => CrosstermColor::DarkBlue,
        RsColor::BrightMagenta => CrosstermColor::Magenta,
        RsColor::Magenta => CrosstermColor::DarkMagenta,
        RsColor::BrightCyan => CrosstermColor::Cyan,
        RsColor::Cyan => CrosstermColor::DarkCyan,
        RsColor::BrightWhite => CrosstermColor::White,
        RsColor::White => CrosstermColor::Grey,
    }
}

/// Render a single row of cells into a `Vec<u8>` that can be printed
fn render_row(row: &[Cell], out: &mut Vec<u8>) {
    // `unwrap` is sprinkled throughout this code, and is safe because we're queueing/writing into a `Vec`,
    // which is an infallible destination for bytes. (barring allocation failure but that's not handled rn anyway.)

    let mut ch_b = [0u8; 4];

    let mut fg = row[0].get_fmt().fg;
    let mut bg = row[0].get_fmt().bg;
    let mut bold = row[0].get_fmt().bold;
    let mut underline = row[0].get_fmt().underline;
    let mut attrs = [Attribute::NormalIntensity, Attribute::NoUnderline];
    if bold {
        attrs[0] = Attribute::Bold;
    }
    if underline {
        attrs[1] = Attribute::Underlined;
    }
    crossterm::queue!(
        out,
        ResetColor,
        SetAttribute(Attribute::Reset),
        SetForegroundColor(ct4rs_color(fg)),
        SetBackgroundColor(ct4rs_color(bg)),
        SetAttributes(attrs.as_ref().into()),
    )
    .unwrap();
    out.extend_from_slice(row[0].ch.encode_utf8(&mut ch_b).as_bytes());

    for cell in &row[1..] {
        if cell.get_fmt().fg != fg {
            fg = cell.get_fmt().fg;
            crossterm::execute!(out, SetForegroundColor(ct4rs_color(fg))).unwrap();
        }
        if cell.get_fmt().bg != bg {
            bg = cell.get_fmt().bg;
            crossterm::execute!(out, SetBackgroundColor(ct4rs_color(bg))).unwrap();
        }
        if cell.get_fmt().bold != bold {
            bold = cell.get_fmt().bold;
            let attr = if bold {
                Attribute::Bold
            } else {
                Attribute::NormalIntensity
            };
            crossterm::execute!(out, SetAttribute(attr)).unwrap();
        }
        if cell.get_fmt().underline != underline {
            underline = cell.get_fmt().underline;
            let attr = if underline {
                Attribute::Underlined
            } else {
                Attribute::NoUnderline
            };
            crossterm::execute!(out, SetAttribute(attr)).unwrap();
        }
        out.extend_from_slice(cell.ch.encode_utf8(&mut ch_b).as_bytes());
    }
    crossterm::execute!(out, MoveDown(1), MoveToColumn(0)).unwrap();
}

pub struct CtSystem {
    queue: mpsc::Receiver<Action>,
    stop: Arc<AtomicBool>,
}

impl CtSystem {
    pub fn new() -> crate::Result<(Self, CtRunner)> {
        let (queue_s, queue_r) = mpsc::channel();
        let stop = Arc::new(AtomicBool::new(false));
        let runner = CtRunner::new(queue_s, stop.clone())?;
        Ok((
            Self {
                queue: queue_r,
                stop: stop.clone(),
            },
            runner,
        ))
    }
}

impl IoSystem for CtSystem {
    fn size(&self) -> XY {
        let (x, y) = terminal::size().unwrap();
        XY(x as usize, y as usize)
    }

    fn draw(&mut self, screen: &Screen) -> crate::Result<()> {
        let mut out = vec![];
        crossterm::queue!(&mut out, MoveTo(0, 0), Clear(ClearType::All)).unwrap();
        for row in screen.rows() {
            render_row(row, &mut out);
        }
        let stdout = std::io::stdout();
        let mut stdout = stdout.lock();
        stdout.write_all(&out)?;
        stdout.flush()?;
        Ok(())
    }

    fn input(&mut self) -> crate::Result<Action> {
        Ok(self.queue.recv().expect("unexpected queue closure"))
    }

    fn poll_input(&mut self) -> crate::Result<Option<Action>> {
        match self.queue.try_recv() {
            Ok(res) => Ok(Some(res)),
            Err(TryRecvError::Disconnected) => panic!("unexpected queue closure"),
            Err(TryRecvError::Empty) => Ok(None),
        }
    }

    fn stop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        while self.queue.recv() != Err(RecvError) {
            // flushing the queue and waiting for it to disconnect in the condition itself
        }
    }
}
