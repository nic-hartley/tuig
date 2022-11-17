use std::io;

use crate::io::sys::IoSystem;

mod intro;

pub async fn intro(io: &mut dyn IoSystem) -> io::Result<()> {
    intro::Intro::new(io).run().await
}
