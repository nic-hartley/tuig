//! This module provides input adapters for each of the various I/O mechanisms supported. Each one is controlled by
//! a feature named similarly and exports a struct implementing `IoSystem`. The actual intended input and output APIs
//! are in the `input` and `output` modules.

use std::io;

use super::{input::Action, output::Screen, XY};

mod ansi_cli;

#[async_trait::async_trait]
pub trait IoSystem {
    async fn draw(&mut self, screen: &Screen) -> io::Result<()>;
    fn size(&self) -> XY;
    async fn input(&mut self) -> io::Result<Action>;
}

pub fn load() -> Box<dyn IoSystem> {
    Box::new(ansi_cli::AnsiScreen::get().unwrap())
}
