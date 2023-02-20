//! This module provides input adapters for each of the various I/O mechanisms supported. Each one is controlled by
//! a feature named similarly and exports a struct implementing `IoSystem`. The actual intended input and output APIs
//! are in the `input` and `output` modules.

#[cfg(feature = "__sys")]
use std::collections::HashMap;
use std::{
    io,
    time::{Duration, Instant},
};

use super::{input::Action, output::Screen, XY};

#[cfg(feature = "sys_cli")]
pub mod ansi_cli;

#[cfg(feature = "__sys_gui")]
pub mod gui;

#[cfg(feature = "sys_nop")]
pub mod nop;

/// An input/output system.
///
/// The output is called a "display" to distinguish it from the [`Screen`].
///
/// This object is meant to be associated with a [`IoRunner`], which will run infinitely on the main thread while this
/// is called from within the event system.
pub trait IoSystem: Send {
    /// Actually render a [`Screen`] to the display.
    fn draw(&mut self, screen: &Screen) -> io::Result<()>;
    /// Get the size of the display, in characters.
    fn size(&self) -> XY;

    /// Wait for the next user input.
    fn input(&mut self) -> io::Result<Action>;
    /// If the next user input is available, return it.
    fn poll_input(&mut self) -> io::Result<Option<Action>>;
    /// Wait for the next user input, up to a timeout.
    fn input_until(&mut self, time: Duration) -> io::Result<Option<Action>> {
        let end = Instant::now() + time;
        while Instant::now() < end {
            if let Some(input) = self.poll_input()? {
                return Ok(Some(input));
            }
            std::thread::sleep(time / 20);
        }
        Ok(None)
    }

    /// Tells the associated [`IoRunner`] to stop and return control of the main thread, and tell the [`IoSystem`] to
    /// dispose of any resources it's handling.
    ///
    /// This will always be the last method called on this object (unless you count `Drop::drop`) so feel free to
    /// panic in the others if they're called after this one, especially `draw`.
    fn stop(&mut self);
}

/// The other half of an [`IoSystem`].
///
/// This type exists so that things which need to run on the main thread specifically, can.
pub trait IoRunner {
    /// Execute one 'step', which should be quick and must be non-blocking. Returns whether an exit has been requested
    /// (i.e. by [`IoSystem::stop`]) since the last time `step` was called.
    ///
    /// Will always be called on the main thread.
    #[must_use]
    fn step(&mut self) -> bool;

    /// Run until the paired [`IoSystem`] tells you to stop.
    ///
    /// Will always be called on the main thread.
    ///
    /// The default implementation just runs `while !self.step() { }`.
    fn run(&mut self) {
        while !self.step() {}
    }
}
/// Based on IO system features enabled, attempt to initialize an IO system; in order:
///
/// - NOP (`nop`), for benchmarks
/// - Vulkan GUI (`gui_vulkan`)
/// - OpenGL GUI (`gui_opengl`)
/// - CPU-rendered GUI (`gui_cpu`)
/// - crossterm CLI (`cli_crossterm`)
///
/// The Err type is a map from the name of the system (in code formatting above) to the error that it hit.
#[cfg(feature = "__sys")]
pub fn load() -> Result<(Box<dyn IoSystem>, Box<dyn IoRunner>), HashMap<&'static str, io::Error>> {
    let mut errors = HashMap::new();
    macro_rules! try_init {
        ( $name:ident: $( $init:tt )* ) => {
            let res = {
                $($init)*
            };
            match res {
                Ok((iosys, run)) => return Ok((Box::new(iosys), Box::new(run))),
                Err(e) => errors.insert(stringify!($name), e),
            };
        }
    }
    #[cfg(feature = "sys_nop")]
    {
        try_init! { nop: nop::NopSystem::new() }
    }
    #[cfg(feature = "__sys_gui")]
    {
        use crate::io::sys::gui::Gui;
        #[cfg(feature = "sys_gui_softbuffer")]
        {
            use crate::io::sys::gui::softbuffer::SoftbufferBackend;
            // Try to initialize softbuffer rendering
            try_init! { softbuffer_gui: Gui::<SoftbufferBackend>::new(20.0) }
        }
    }
    #[cfg(feature = "sys_cli")]
    {
        // Try to initialize the CLI renderer
        try_init! { ansi_cli: ansi_cli::AnsiIo::get() }
    }
    Err(errors)
}
