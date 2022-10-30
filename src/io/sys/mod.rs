//! This module provides input adapters for each of the various I/O mechanisms supported. Each one is controlled by
//! a feature named similarly and exports a struct implementing `IoSystem`. The actual intended input and output APIs
//! are in the `input` and `output` modules.

use std::{collections::HashMap, io};

use super::{input::Action, output::Screen, XY};

#[cfg(feature = "sys_cli")]
mod ansi_cli;

#[async_trait::async_trait]
pub trait IoSystem {
    async fn draw(&mut self, screen: &Screen) -> io::Result<()>;
    fn size(&self) -> XY;
    async fn input(&mut self) -> io::Result<Action>;
}

/// Based on IO system features enabled, attempt to initialize an IO system; in order:
///
/// - Vulkan GUI (`gui_vulkan`)
/// - OpenGL GUI (`gui_opengl`)
/// - CPU-rendered GUI (`gui_cpu`)
/// - crossterm CLI (`cli_crossterm`)
///
/// If none are enabled, this will immediately return `Err(HashMap::new())`.
///
/// The Err type is a map from the name of the system (in code formatting above) to the error that it hit.
pub fn load() -> Result<Box<dyn IoSystem>, HashMap<&'static str, io::Error>> {
    let mut errors = HashMap::new();
    macro_rules! try_init {
        ( $name:ident: $( $init:tt )* ) => {
            let res = || {
                $($init)*
            };
            match res() {
                Ok(res) => return Ok(Box::new(res)),
                Err(e) => errors.insert(stringify!($name), e),
            };
        }
    }
    #[cfg(feature = "sys_gui")]
    {
        // TODO: Try to initialize common GUI components
        #[cfg(feature = "sys_gui_vulkan")]
        {
            // TODO: Try to initialize Vulkan rendering
        }
        #[cfg(feature = "sys_gui_opengl")]
        {
            // TODO: Try to initialize OpenGL rendering
        }
        #[cfg(feature = "sys_gui_cpu")]
        {
            // TODO: Try to initialize CPU rendering
        }
    }
    #[cfg(feature = "sys_cli")]
    {
        // Try to initialize the CLI renderer
        try_init! { ansi_cli: ansi_cli::AnsiScreen::get() }
    }
    // No sys_test initialization: as the name implies it's wholely meant for tests, so it makes basically no sense to
    // try to guess what particular testing input or output will be wanted
    Err(errors)
}

/// Convenience wrapper for [`load`]. If that fails, prints out a little message explaining what went wrong and exits.
/// Meant to be used in concept art, not the actual game.
pub fn load_or_die() -> Box<dyn IoSystem> {
    let errs = match load() {
        Ok(io) => return io,
        Err(e) => e,
    };

    if errs.is_empty() {
        println!("No renderers enabled! Someone compiled this wrong.")
    } else {
        println!("{} renderers tried to load:", errs.len());
        for (name, err) in errs {
            println!("- {}: {:?}", name, err);
        }
    }

    std::process::exit(1);
}
