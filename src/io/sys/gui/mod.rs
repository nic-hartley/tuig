use std::{io, thread, time::{Instant, Duration}};

use tokio::sync::{mpsc, oneshot};
use winit::{window::{Window, WindowBuilder}, event_loop::EventLoopBuilder, dpi::LogicalSize, event::{Event, WindowEvent, ElementState, VirtualKeyCode}, platform::{run_return::EventLoopExtRunReturn, unix::EventLoopBuilderExtUnix}};

use crate::io::{output::Screen, XY, input::{Action, Key}};

use super::IoSystem;

const FONT_TTF: &[u8] = include_bytes!("inconsolata.ttf");

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

fn char4pixel_pos(pos: XY, char_size: XY, win_size: XY) -> XY {
    // buffer around the edges
    let buf = (win_size % char_size) / 2;
    (pos - buf) / char_size
}

async fn spawn_window(char_size: XY, win_size: XY) -> io::Result<(Window, mpsc::UnboundedReceiver<Action>, oneshot::Sender<()>)> {
    let (win_send, win_recv) = oneshot::channel();
    let (act_send, act_recv) = mpsc::unbounded_channel();
    let (kill_send, mut kill_recv) = oneshot::channel();
    thread::spawn(move || {
        let mut el = EventLoopBuilder::<Action>::with_user_event()
            .with_x11()
            .with_any_thread(true)
            .build();
        let window = WindowBuilder::new()
            .with_inner_size(LogicalSize::new(win_size.x() as u32, win_size.y() as u32))
            .with_resizable(false)
            .build(&el);
        win_send.send(window).expect("failed to send initialized window to main thread");

        // the bugs don't bother us anyway -- we just don't want the entire process to exit when this is done.
        el.run_return(move |ev, _, cf| {
            match kill_recv.try_recv() {
                Err(oneshot::error::TryRecvError::Empty) => (),
                _ => {
                    cf.set_exit();
                    return;
                }
            }

            // ensure that we check around once a second to see if we should quit
            cf.set_wait_until(Instant::now() + Duration::from_secs(1));

            let actions: [Option<Action>; 4];
            macro_rules! set {
                ( @@impl
                    $( $a1:expr, $( $a2:expr, $( $a3:expr, $( $a4:expr, $( None, )* )? )? )? )?
                ) => {
                    actions = [
                        $( $a1, $( $a2, $( $a3, $( $a4, )? )? )? )?
                    ]
                };
                ( $( $act:expr ),* $(,)? ) => {
                    set!(@@impl $( Some($act), )* None, None, None, None, )
                };
            }
            match ev {
                Event::UserEvent(a) => set!(a),
                Event::WindowEvent { event: WindowEvent::Resized(_) | WindowEvent::Moved(_), .. } => {
                    set!(Action::Resized);
                }
                Event::WindowEvent { event: WindowEvent::CloseRequested | WindowEvent::Destroyed, .. } => {
                    set!(Action::Closed);
                }
                Event::WindowEvent { event: WindowEvent::KeyboardInput { input, .. }, .. } => {
                    match key4vkc(input.virtual_keycode) {
                        Some(key) => match input.state {
                            ElementState::Pressed => set!(Action::KeyPress { key }),
                            ElementState::Released => set!(Action::KeyRelease { key }),
                        }
                        // not a key we care about
                        None => return,
                    }
                }
                // TODO: Enable and handle IME -- useful for folks with compose keys
                // for now this is good enough
                Event::WindowEvent { event: WindowEvent::ReceivedCharacter(ch), .. } => {
                    set!(Action::KeyPress { key: Key::Char(ch) }, Action::KeyRelease { key: Key::Char(ch) });
                }
                Event::WindowEvent { event: WindowEvent::CursorMoved { position, .. }, .. } => {
                    let position = XY(position.x as usize, position.y as usize);
                    set!(Action::MouseMove { pos: char4pixel_pos(position, char_size, win_size) });
                }
                // TODO: Handle other mouse events

                // other things can be ignored (for now)
                _ => return,
            };

            for action in actions {
                if let Some(action) = action {
                    if let Err(_) = act_send.send(action) {
                        // other end already hung up, wtaf
                        cf.set_exit();
                        break;
                    }
                }
            }
        });
    });

    let win = win_recv.await
        .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "event loop thread crashed while starting"))?
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    Ok((win, act_recv, kill_send))
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
    kill_el: Option<oneshot::Sender<()>>,
    backend: B,
}

impl<B: GuiBackend> Gui<B> {
    pub async fn new(font_size: f32) -> io::Result<Self> {
        let backend = B::new(font_size)?;
        let char_size = backend.char_size();
        let win_size = char_size * XY(80, 25);
        let (window, inputs, kill_el) = spawn_window(char_size, win_size).await?;
        Ok(Self { window, inputs, kill_el: Some(kill_el), backend })
    }
}

#[async_trait::async_trait]
impl<B: GuiBackend> IoSystem for Gui<B> {
    async fn draw(&mut self, screen: &Screen) -> io::Result<()> {
        self.backend.render(&self.window, screen).await?;
        self.window.request_redraw();
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
        self.inputs.recv().await
            .ok_or(io::Error::new(io::ErrorKind::BrokenPipe, "input loop has terminated unexpectedly"))
    }
}

impl<B: GuiBackend> Drop for Gui<B> {
    fn drop(&mut self) {
        if let Some(send) = self.kill_el.take() {
            // ignore errors -- nothing we could do about it anyway
            let _ = send.send(());
        }
    }
}
