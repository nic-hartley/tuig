use std::{mem, time::Duration, io::stdout};

use crossterm::{event as ct, terminal, execute};
use crate::io::input as io;

use tokio::{sync::{mpsc, oneshot}, task::spawn_blocking};

use super::{Action, Input, Key};

macro_rules! mods {
    ( $mods:ident, $action:ident ) => {
        if $mods.contains(ct::KeyModifiers::SHIFT) {
            try_send!($action { key: Key::LeftShift });
        }
        if $mods.contains(ct::KeyModifiers::CONTROL) {
            try_send!($action { key: Key::LeftCtrl });
        }
        if $mods.contains(ct::KeyModifiers::ALT) {
            try_send!($action { key: Key::LeftAlt });
        }
    }
}

fn io4ct_btn(ct: ct::MouseButton) -> io::MouseButton {
    match ct {
        ct::MouseButton::Left => io::MouseButton::Left,
        ct::MouseButton::Middle => io::MouseButton::Middle,
        ct::MouseButton::Right => io::MouseButton::Right,
    }
}

fn process_input(actions: mpsc::UnboundedSender<Action>, mut stop: oneshot::Receiver<()>) {
    macro_rules! try_send {
        ( $type:ident $( ($nt:expr) )? $( { $($br:tt)* } )? ) => {
            match actions.send(io::Action::$type $(($nt))? $({$($br)*})? ) {
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
            },
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
                mods! (modifiers, KeyPress);
                if code == ct::KeyCode::BackTab {
                    try_send!(KeyPress { key: Key::LeftShift });
                    try_send!(KeyPress { key: Key::Tab });
                    try_send!(KeyRelease { key: Key::Tab });
                    try_send!(KeyRelease { key: Key::LeftShift });
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
                mods! (modifiers, KeyRelease);
            }
            ct::Event::Resize(..) => todo!("Handle resize events properly"),
            ct::Event::Mouse(ct::MouseEvent { row, column: col, kind, modifiers }) => {
                mods!(modifiers, KeyPress);
                let pos = io::XY(col as usize, row as usize);
                match kind {
                    ct::MouseEventKind::Up(btn) => try_send!(MouseRelease { button: io4ct_btn(btn), pos }),
                    ct::MouseEventKind::Down(btn) => try_send!(MousePress { button: io4ct_btn(btn), pos }),
                    ct::MouseEventKind::Drag(btn) => try_send!(MouseMove { button: Some(io4ct_btn(btn)), pos }),
                    ct::MouseEventKind::Moved => try_send!(MouseMove { button: None, pos }),
                    ct::MouseEventKind::ScrollUp => {
                        try_send!(MousePress { button: io::MouseButton::ScrollUp, pos });
                        try_send!(MousePress { button: io::MouseButton::ScrollUp, pos });
                    }
                    ct::MouseEventKind::ScrollDown => {
                        try_send!(MousePress { button: io::MouseButton::ScrollDown, pos });
                        try_send!(MousePress { button: io::MouseButton::ScrollDown, pos });
                    }
                }
                mods!(modifiers, KeyRelease);
            }
        };
    }
}

pub struct AnsiInput {
    queue: mpsc::UnboundedReceiver<Action>,
    stop: Option<oneshot::Sender<()>>,
}

impl AnsiInput {
    fn init_term() -> crossterm::Result<()> {
        terminal::enable_raw_mode()?;
        execute!(stdout(), ct::EnableMouseCapture)?;
        Ok(())
    }

    fn cleanup_term() {
        let _ = execute!(stdout(), ct::DisableMouseCapture);
        let _ = terminal::disable_raw_mode();
    }

    pub fn get() -> crossterm::Result<AnsiInput> {
        Self::init_term()?;
        std::panic::set_hook(Box::new(|_| Self::cleanup_term()));
        let (queue_s, queue_r) = mpsc::unbounded_channel();
        let (stop_s, stop_r) = oneshot::channel();
        spawn_blocking(|| process_input(queue_s, stop_r));
        Ok(AnsiInput { queue: queue_r, stop: Some(stop_s) })
    }
}

#[async_trait::async_trait]
impl Input for AnsiInput {
    async fn next(&mut self) -> Action {
        self.queue.recv().await.expect("Queue should not empty and die before we stop due to an error")
    }

    fn flush(&mut self) {
        while let Ok(_) = self.queue.try_recv() {
            // already pulled the thing out, nothing to do with it
        }
    }
}

impl Drop for AnsiInput {
    fn drop(&mut self) {
        let stop = mem::replace(&mut self.stop, None).expect("Tried to drop an already dropped AnsiInput");
        // if the receiver is dead, that's fine; that means the queue won't have anything
        // else put in it. (and it can happen if e.g. there's an IO error)
        let _ = stop.send(());
        while self.queue.try_recv() != Err(mpsc::error::TryRecvError::Disconnected) {
            // flushing the queue and waiting for it to disconnect in the condition itself
        }
        Self::cleanup_term();
    }
}
