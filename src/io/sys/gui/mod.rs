use std::{
    io,
    sync::{Arc, Once},
    time::{Duration, Instant},
};

use tokio::sync::mpsc;
use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event, VirtualKeyCode, WindowEvent},
    event_loop::{EventLoop, EventLoopBuilder},
    platform::run_return::EventLoopExtRunReturn,
    window::{Window, WindowBuilder},
};

use crate::io::{
    input::{Action, Key, MouseButton},
    output::Screen,
    XY,
};

use super::{IoRunner, IoSystem};

const REGULAR_TTF: &[u8] = include_bytes!("inconsolata-reg.ttf");
const BOLD_TTF: &[u8] = include_bytes!("inconsolata-bold.ttf");

fn key4vkc(vkc: Option<VirtualKeyCode>) -> Option<Key> {
    match vkc? {
        VirtualKeyCode::Key1 => Some(Key::Char('1')),
        VirtualKeyCode::Key2 => Some(Key::Char('2')),
        VirtualKeyCode::Key3 => Some(Key::Char('3')),
        VirtualKeyCode::Key4 => Some(Key::Char('4')),
        VirtualKeyCode::Key5 => Some(Key::Char('5')),
        VirtualKeyCode::Key6 => Some(Key::Char('6')),
        VirtualKeyCode::Key7 => Some(Key::Char('7')),
        VirtualKeyCode::Key8 => Some(Key::Char('8')),
        VirtualKeyCode::Key9 => Some(Key::Char('9')),
        VirtualKeyCode::Key0 => Some(Key::Char('0')),
        VirtualKeyCode::A => Some(Key::Char('a')),
        VirtualKeyCode::B => Some(Key::Char('b')),
        VirtualKeyCode::C => Some(Key::Char('c')),
        VirtualKeyCode::D => Some(Key::Char('d')),
        VirtualKeyCode::E => Some(Key::Char('e')),
        VirtualKeyCode::F => Some(Key::Char('f')),
        VirtualKeyCode::G => Some(Key::Char('g')),
        VirtualKeyCode::H => Some(Key::Char('h')),
        VirtualKeyCode::I => Some(Key::Char('i')),
        VirtualKeyCode::J => Some(Key::Char('j')),
        VirtualKeyCode::K => Some(Key::Char('k')),
        VirtualKeyCode::L => Some(Key::Char('l')),
        VirtualKeyCode::M => Some(Key::Char('m')),
        VirtualKeyCode::N => Some(Key::Char('n')),
        VirtualKeyCode::O => Some(Key::Char('o')),
        VirtualKeyCode::P => Some(Key::Char('p')),
        VirtualKeyCode::Q => Some(Key::Char('q')),
        VirtualKeyCode::R => Some(Key::Char('r')),
        VirtualKeyCode::S => Some(Key::Char('s')),
        VirtualKeyCode::T => Some(Key::Char('t')),
        VirtualKeyCode::U => Some(Key::Char('u')),
        VirtualKeyCode::V => Some(Key::Char('v')),
        VirtualKeyCode::W => Some(Key::Char('w')),
        VirtualKeyCode::X => Some(Key::Char('x')),
        VirtualKeyCode::Y => Some(Key::Char('y')),
        VirtualKeyCode::Z => Some(Key::Char('z')),
        VirtualKeyCode::Escape => Some(Key::Escape),
        VirtualKeyCode::F1 => Some(Key::F(1)),
        VirtualKeyCode::F2 => Some(Key::F(2)),
        VirtualKeyCode::F3 => Some(Key::F(3)),
        VirtualKeyCode::F4 => Some(Key::F(4)),
        VirtualKeyCode::F5 => Some(Key::F(5)),
        VirtualKeyCode::F6 => Some(Key::F(6)),
        VirtualKeyCode::F7 => Some(Key::F(7)),
        VirtualKeyCode::F8 => Some(Key::F(8)),
        VirtualKeyCode::F9 => Some(Key::F(9)),
        VirtualKeyCode::F10 => Some(Key::F(10)),
        VirtualKeyCode::F11 => Some(Key::F(11)),
        VirtualKeyCode::F12 => Some(Key::F(12)),
        VirtualKeyCode::F13 => Some(Key::F(13)),
        VirtualKeyCode::F14 => Some(Key::F(14)),
        VirtualKeyCode::F15 => Some(Key::F(15)),
        VirtualKeyCode::F16 => Some(Key::F(16)),
        VirtualKeyCode::F17 => Some(Key::F(17)),
        VirtualKeyCode::F18 => Some(Key::F(18)),
        VirtualKeyCode::F19 => Some(Key::F(19)),
        VirtualKeyCode::F20 => Some(Key::F(20)),
        VirtualKeyCode::F21 => Some(Key::F(21)),
        VirtualKeyCode::F22 => Some(Key::F(22)),
        VirtualKeyCode::F23 => Some(Key::F(23)),
        VirtualKeyCode::F24 => Some(Key::F(24)),
        VirtualKeyCode::Insert => Some(Key::Insert),
        VirtualKeyCode::Home => Some(Key::Home),
        VirtualKeyCode::Delete => Some(Key::Delete),
        VirtualKeyCode::End => Some(Key::End),
        VirtualKeyCode::PageDown => Some(Key::PageDown),
        VirtualKeyCode::PageUp => Some(Key::PageUp),
        VirtualKeyCode::Left => Some(Key::Left),
        VirtualKeyCode::Up => Some(Key::Up),
        VirtualKeyCode::Right => Some(Key::Right),
        VirtualKeyCode::Down => Some(Key::Down),
        VirtualKeyCode::Back => Some(Key::Backspace),
        VirtualKeyCode::Return => Some(Key::Enter),
        VirtualKeyCode::Space => Some(Key::Char(' ')),
        VirtualKeyCode::Caret => Some(Key::Char('^')),
        VirtualKeyCode::Numpad0 => Some(Key::Char('0')),
        VirtualKeyCode::Numpad1 => Some(Key::Char('1')),
        VirtualKeyCode::Numpad2 => Some(Key::Char('2')),
        VirtualKeyCode::Numpad3 => Some(Key::Char('3')),
        VirtualKeyCode::Numpad4 => Some(Key::Char('4')),
        VirtualKeyCode::Numpad5 => Some(Key::Char('5')),
        VirtualKeyCode::Numpad6 => Some(Key::Char('6')),
        VirtualKeyCode::Numpad7 => Some(Key::Char('7')),
        VirtualKeyCode::Numpad8 => Some(Key::Char('8')),
        VirtualKeyCode::Numpad9 => Some(Key::Char('9')),
        VirtualKeyCode::NumpadAdd => Some(Key::Char('+')),
        VirtualKeyCode::NumpadDivide => Some(Key::Char('/')),
        VirtualKeyCode::NumpadDecimal => Some(Key::Char('.')),
        VirtualKeyCode::NumpadComma => Some(Key::Char(',')),
        VirtualKeyCode::NumpadEnter => Some(Key::Enter),
        VirtualKeyCode::NumpadEquals => Some(Key::Char('=')),
        VirtualKeyCode::NumpadMultiply => Some(Key::Char('*')),
        VirtualKeyCode::NumpadSubtract => Some(Key::Char('-')),
        VirtualKeyCode::Apostrophe => Some(Key::Char('\'')),
        VirtualKeyCode::Asterisk => Some(Key::Char('*')),
        VirtualKeyCode::At => Some(Key::Char('@')),
        VirtualKeyCode::Backslash => Some(Key::Char('\\')),
        VirtualKeyCode::Colon => Some(Key::Char(':')),
        VirtualKeyCode::Comma => Some(Key::Char(',')),
        VirtualKeyCode::Equals => Some(Key::Char('=')),
        VirtualKeyCode::Grave => Some(Key::Char('`')),
        VirtualKeyCode::LAlt => Some(Key::LeftAlt),
        VirtualKeyCode::LBracket => Some(Key::Char('[')),
        VirtualKeyCode::LControl => Some(Key::LeftCtrl),
        VirtualKeyCode::LShift => Some(Key::LeftShift),
        VirtualKeyCode::LWin => Some(Key::LeftSuper),
        VirtualKeyCode::Minus => Some(Key::Char('-')),
        VirtualKeyCode::Period => Some(Key::Char('.')),
        VirtualKeyCode::Plus => Some(Key::Char('+')),
        VirtualKeyCode::RAlt => Some(Key::RightAlt),
        VirtualKeyCode::RBracket => Some(Key::Char(']')),
        VirtualKeyCode::RControl => Some(Key::RightCtrl),
        VirtualKeyCode::RShift => Some(Key::RightShift),
        VirtualKeyCode::RWin => Some(Key::RightSuper),
        VirtualKeyCode::Semicolon => Some(Key::Char(';')),
        VirtualKeyCode::Slash => Some(Key::Char('/')),
        VirtualKeyCode::Tab => Some(Key::Tab),
        // TODO: Figure out
        // - VirtualKeyCode::Ax
        // - VirtualKeyCode::Capital
        // - VirtualKeyCode::Convert
        // - VirtualKeyCode::NavigateForward
        // - VirtualKeyCode::NavigateBackward
        // - VirtualKeyCode::NoConvert
        // - VirtualKeyCode::Unlabeled
        _ => None,
    }
}

fn mb4button(button: winit::event::MouseButton) -> Option<MouseButton> {
    match button {
        winit::event::MouseButton::Left => Some(MouseButton::Left),
        winit::event::MouseButton::Middle => Some(MouseButton::Middle),
        winit::event::MouseButton::Right => Some(MouseButton::Right),
        winit::event::MouseButton::Other(_) => None,
    }
}

fn char4pixel_pos(pos: XY, char_size: XY, win_size: XY) -> XY {
    // buffer around the edges
    let buf = (win_size % char_size) / 2;
    let pos = pos.clamp(buf, win_size - char_size + buf);
    (pos - buf) / char_size
}

struct WindowSpawnOutput {
    window: Window,
    action_recv: mpsc::UnboundedReceiver<Action>,
    kill_send: Arc<Once>,
    runner: WindowRunner,
}

fn spawn_window(char_size: XY, win_size: XY) -> io::Result<WindowSpawnOutput> {
    let el = EventLoopBuilder::<Action>::with_user_event().build();
    let window = WindowBuilder::new()
        .with_inner_size(LogicalSize::new(win_size.x() as u32, win_size.y() as u32))
        .with_title("redshell")
        .build(&el)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    let (act_send, action_recv) = mpsc::unbounded_channel();

    let killer = Arc::new(Once::new());
    let kill_recv = killer.clone();
    let kill_send = killer.clone();
    let runner = WindowRunner {
        el,
        act_send,
        kill_recv,
        char_size,
        win_size,
        prev_cursor_pos: XY(0, 0),
    };
    Ok(WindowSpawnOutput {
        window,
        action_recv,
        kill_send,
        runner,
    })
}

#[async_trait::async_trait]
pub trait GuiBackend: Send + Sync + Sized {
    /// Create a new rendering backend with the given font size. The font size is `fontdue`'s understanding of it: The
    /// (approximate) width of the `m` character. In Inconsolata, or really any monospace font, that should also be
    /// the width of every other character.
    fn new(font_size: f32) -> io::Result<Self>;

    /// Reset the renderer to use a new font size.
    ///
    /// The default implementation simply destroys the old renderer and replaces it in-place with a new one, but there
    /// may be more efficient implementations for any given backend.
    ///
    /// This doesn't need to re-render anything, but it cannot break the current window.
    fn renew(&mut self, font_size: f32) -> io::Result<()> {
        let new = Self::new(font_size)?;
        *self = new;
        Ok(())
    }

    /// Render a screen onto the window.
    ///
    /// This must only return when the rendering is as definitively complete as the backend can easily determine.
    async fn render(&self, window: &Window, screen: &Screen) -> io::Result<()>;

    /// Return the bounding box dimensions of the characters being used in the font being used.
    fn char_size(&self) -> XY;
}

pub struct Gui<B: GuiBackend> {
    window: Window,
    inputs: mpsc::UnboundedReceiver<Action>,
    kill_el: Arc<Once>,
    backend: B,
}

impl<B: GuiBackend> Gui<B> {
    pub fn new(font_size: f32) -> io::Result<(Self, WindowRunner)> {
        let backend = B::new(font_size)?;
        let char_size = backend.char_size();
        let win_size = char_size * XY(80, 25);
        let WindowSpawnOutput {
            window,
            action_recv: inputs,
            kill_send,
            runner,
        } = spawn_window(char_size, win_size)?;
        Ok((
            Self {
                window,
                inputs,
                kill_el: kill_send,
                backend,
            },
            runner,
        ))
    }
}

#[async_trait::async_trait]
impl<B: GuiBackend> IoSystem for Gui<B> {
    async fn draw(&mut self, screen: &Screen) -> io::Result<()> {
        self.backend.render(&self.window, screen).await?;
        Ok(())
    }

    fn size(&self) -> XY {
        let raw_sz = self.window.inner_size();
        let char_sz = self.backend.char_size();
        let width = raw_sz.width as usize / char_sz.x();
        let height = raw_sz.height as usize / char_sz.y();
        XY(width, height)
    }

    async fn input(&mut self) -> io::Result<Action> {
        self.inputs.recv().await.ok_or(io::Error::new(
            io::ErrorKind::BrokenPipe,
            "input loop has terminated unexpectedly",
        ))
    }

    fn stop(&mut self) {
        self.kill_el.call_once(|| {})
    }
}

pub struct WindowRunner {
    el: EventLoop<Action>,
    act_send: mpsc::UnboundedSender<Action>,
    kill_recv: Arc<Once>,
    char_size: XY,
    win_size: XY,
    prev_cursor_pos: XY,
}

impl IoRunner for WindowRunner {
    fn run(&mut self) {
        // the bugs don't bother us anyway -- we just don't want the entire process to exit when this is done.
        self.el.run_return(|ev, _, cf| {
            if self.kill_recv.is_completed() {
                cf.set_exit();
                return;
            }

            // ensure that we check at least once a second to see if we should quit
            cf.set_wait_until(Instant::now() + Duration::from_secs(1));

            macro_rules! send {
                ( $( $act:expr ),* $(,)? ) => { {
                    $(
                        if let Err(_) = self.act_send.send( $act ) {
                            // they hung up on us! how rude!
                            cf.set_exit();
                        }
                    )*
                } };
            }
            match ev {
                Event::UserEvent(a) => send!(a),
                Event::WindowEvent {
                    event: WindowEvent::Resized(sz),
                    ..
                } => {
                    self.win_size = XY(sz.width as usize, sz.height as usize);
                    send!(Action::Resized);
                }
                Event::RedrawRequested(_) => send!(Action::Resized),
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested | WindowEvent::Destroyed,
                    ..
                } => {
                    send!(Action::Closed);
                }
                // TODO: Enable and handle IME -- useful for folks with compose keys
                // for now this is good enough
                Event::WindowEvent {
                    event: WindowEvent::KeyboardInput { input, .. },
                    ..
                } => {
                    if let Some(key) = key4vkc(input.virtual_keycode) {
                        match input.state {
                            ElementState::Pressed => send!(Action::KeyPress { key }),
                            ElementState::Released => send!(Action::KeyRelease { key }),
                        }
                    }
                }
                Event::WindowEvent {
                    event: WindowEvent::CursorMoved { position, .. },
                    ..
                } => {
                    let position = XY(position.x as usize, position.y as usize);
                    let position = char4pixel_pos(position, self.char_size, self.win_size);
                    if self.prev_cursor_pos != position {
                        send!(Action::MouseMove { pos: position });
                        self.prev_cursor_pos = position;
                    }
                }
                Event::WindowEvent {
                    event: WindowEvent::MouseInput { state, button, .. },
                    ..
                } => {
                    if let Some(button) = mb4button(button) {
                        match state {
                            ElementState::Pressed => send!(Action::MousePress { button }),
                            ElementState::Released => send!(Action::MouseRelease { button }),
                        }
                    }
                }
                Event::Suspended => send!(Action::Paused),
                Event::Resumed => send!(Action::Unpaused),

                // other things can be ignored (for now)
                _ => (),
            };
        });
    }
}

#[cfg(feature = "sys_gui_softbuffer")]
pub mod softbuffer;
