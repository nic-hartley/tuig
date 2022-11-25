use std::{io, time::Duration};

use crate::io::{
        output::Screen,
        sys::IoSystem,
    };

mod splatty;
pub use splatty::*;

mod tutorial1;
pub use tutorial1::*;

async fn sleep(s: f32) {
    tokio::time::sleep(Duration::from_secs_f32(s)).await
}

pub async fn run(io: &mut dyn IoSystem) -> io::Result<String> {
    let mut screen = Screen::new(io.size());

    // TODO: Rewrite these functions to use methods and fields instead of macros and variables

    let seed = sprinkler_wave(io, &mut screen).await?;
    loading_text(io, &mut screen, seed).await?;

    // screen should now be blank so we can start on the actual intro
    let name = tutorial(io, &mut screen).await?;

    Ok(name)
}
